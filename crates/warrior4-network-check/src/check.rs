use std::{fmt::Display, time::Duration};

use hickory_resolver::{
    TokioResolver,
    net::{DnsError, NetError},
    proto::op::ResponseCode,
};
use rand::distr::{Alphanumeric, SampleString};
use reqwest::Client;

use crate::{adapter::DnsClientAdapter, config::TargetConfig};

#[derive(Debug, Default)]
pub enum TestResult {
    #[default]
    Incomplete,
    Pass,
    Fail(String),
    Error(Box<dyn std::error::Error>),
}

impl TestResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }
}

impl Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestResult::Incomplete => write!(f, "incomplete"),
            TestResult::Pass => write!(f, "pass"),
            TestResult::Fail(message) => write!(f, "fail: {message}"),
            TestResult::Error(error) => write!(f, "error: {error}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Report {
    /// Test of whether we can contact any public resolver.
    pub public_dns_resolver: TestResult,
    /// Test of whether our DNS resolver of choice is rejected.
    pub cleartext_dns_resolver: TestResult,
    /// Test of whether encrypted connection to a DNS resolver is rejected.
    pub encrypted_dns_resolver: TestResult,
    /// Test of the configuration and functionality of the system DNS resolver.
    pub system_dns_resolver: TestResult,
    /// Test of whether the network has a redirection service.
    pub non_existent_domain_cleartext: TestResult,
    /// Test of whether the system resolver is misconfigured with a redirection service.
    pub non_existent_domain_system: TestResult,
    /// Test of whether ISP uses HTML injection.
    pub cleartext_download: TestResult,
    /// Test of whether TLS connections are permitted and authentic.
    pub encrypted_download: TestResult,
}

impl Report {
    pub fn is_pass(&self) -> bool {
        self.cleartext_dns_resolver.is_pass()
            && self.encrypted_dns_resolver.is_pass()
            && self.system_dns_resolver.is_pass()
            && self.non_existent_domain_cleartext.is_pass()
            && self.non_existent_domain_system.is_pass()
            && self.cleartext_download.is_pass()
            && self.encrypted_download.is_pass()
    }
}

#[allow(clippy::result_large_err)]
pub async fn check_network(config: &TargetConfig) -> Result<Report, (Report, std::io::Error)> {
    let mut report = Report::default();
    let public_dns_resolver = crate::builder::build_custom_public_dns_resolver(config);
    let cleartext_dns_resolver = crate::builder::build_custom_cleartext_dns_resolver(config);
    let encrypted_dns_resolver = crate::builder::build_custom_encrypted_dns_resolver(config);
    let system_dns_resolver = crate::builder::build_system_dns_resolver();
    let domain = &config.domain;
    let nonexistent_domain = format_random_domain(&config.nonexistent_domain);
    let cleartext_url = config.url.replace("{protocol}", "http");
    let encrypted_url = config.url.replace("{protocol}", "https");

    eprintln!("Testing DNS resolver ({domain}):");

    eprint!("  Public ... ");
    let result = check_dns_resolver(&public_dns_resolver, domain).await;
    eprintln!("{result}");
    report.public_dns_resolver = result;

    eprint!("  Cleartext ... ");
    let result = check_dns_resolver(&cleartext_dns_resolver, domain).await;
    eprintln!("{result}");
    report.cleartext_dns_resolver = result;

    eprint!("  Encrypted ... ");
    let result = check_dns_resolver(&encrypted_dns_resolver, domain).await;
    eprintln!("{result}");
    report.encrypted_dns_resolver = result;

    eprint!("  System... ");
    let result = check_dns_resolver(&system_dns_resolver, domain).await;
    eprintln!("{result}");
    report.system_dns_resolver = result;

    eprintln!("Testing DNS resolver (non-existent {nonexistent_domain}):");

    eprint!("  Cleartext ... ");
    let result = check_dns_resolver_nx_domain(&cleartext_dns_resolver, &nonexistent_domain).await;
    eprintln!("{result}");
    report.non_existent_domain_cleartext = result;

    eprint!("  System ... ");
    let result = check_dns_resolver_nx_domain(&system_dns_resolver, &nonexistent_domain).await;
    eprintln!("{result}");
    report.non_existent_domain_system = result;

    eprintln!("Checking web page download ({cleartext_url}):");

    eprint!("  Cleartext ... ");
    let result = check_download(
        cleartext_dns_resolver.clone(),
        &cleartext_url,
        &config.url_content,
    )
    .await;
    eprintln!("{result}");
    report.cleartext_download = result;

    eprint!("  Encrypted ... ");
    let result = check_download(
        cleartext_dns_resolver.clone(),
        &encrypted_url,
        &config.url_content,
    )
    .await;
    eprintln!("{result}");
    report.encrypted_download = result;

    Ok(report)
}

/// Generate a random domain name using a template.
fn format_random_domain(template: &str) -> String {
    let chars = Alphanumeric.sample_string(&mut rand::rng(), 16);
    template.replace("{random}", &chars)
}

/// Check whether the domain can be resolved with the given resolver.
async fn check_dns_resolver(resolver: &TokioResolver, domain: &str) -> TestResult {
    let result = resolver.lookup_ip(domain).await;

    match result {
        Ok(_) => TestResult::Pass,
        Err(error) => match error {
            NetError::Dns(dns_error) => TestResult::Fail(dns_error.to_string()),
            _ => TestResult::Error(Box::new(error)),
        },
    }
}

/// Check whether the domain does not exist with the given resolver.
async fn check_dns_resolver_nx_domain(resolver: &TokioResolver, domain: &str) -> TestResult {
    let result = resolver.lookup_ip(domain).await;

    match result {
        Ok(ip_addr) => TestResult::Fail(format!("{ip_addr:?}")),
        Err(NetError::Dns(DnsError::ResponseCode(ResponseCode::NXDomain)))
        | Err(NetError::Dns(DnsError::NoRecordsFound(_))) => TestResult::Pass,
        Err(error) => TestResult::Error(Box::new(error)),
    }
}

/// Check whether the downloaded file matches the expected string.
async fn check_download(resolver: TokioResolver, url: &str, expected_content: &str) -> TestResult {
    let adapter = DnsClientAdapter::new(resolver);
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .dns_resolver(adapter)
        .build()
        .unwrap();

    let result = client.get(url).send().await;

    match result {
        Ok(response) if response.status().is_success() => {
            let content = response.bytes().await.unwrap_or_default();

            if content == expected_content {
                TestResult::Pass
            } else {
                let mut snippet = content.escape_ascii().to_string();
                snippet.truncate(64);
                TestResult::Fail(format!("unexpected content '{snippet}'",))
            }
        }
        Ok(response) => {
            let status_code = response.status().as_u16();
            TestResult::Error(format!("unexpected status code '{status_code}'").into())
        }
        Err(error) => TestResult::Error(Box::new(error)),
    }
}
