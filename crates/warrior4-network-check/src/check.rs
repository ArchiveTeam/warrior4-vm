use std::{fmt::Display, net::SocketAddr, time::Duration};

use dnsclient::{UpstreamServer, sync::DNSClient};
use rand::distr::{Alphanumeric, SampleString};
use ureq::{Agent, unversioned::transport::DefaultConnector};

use crate::{adapter::DnsClientAdapter, config::TargetConfig};

#[derive(Debug)]
pub enum TestResult {
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

impl Default for TestResult {
    fn default() -> Self {
        Self::Incomplete
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
    pub nonexistent: TestResult,
    pub cleartext: TestResult,
    pub target: TestResult,
}

impl Report {
    pub fn is_pass(&self) -> bool {
        self.nonexistent.is_pass() && self.cleartext.is_pass() && self.target.is_pass()
    }
}

pub fn check_network(config: &TargetConfig) -> Result<Report, (Report, std::io::Error)> {
    let mut report = Report::default();
    let custom_client = Agent::with_parts(
        Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .max_redirects(0)
            .build(),
        DefaultConnector::new(),
        DnsClientAdapter::new(custom_dns_client(config)),
    );
    let system_client = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .max_redirects(0)
        .build()
        .new_agent();

    let url = format_random_domain(&config.nonexistent_url);
    eprint!("Check nonexistent resource ({url}) ... ");
    let result = check_nonexistent(&custom_client, &url);
    eprintln!("{result}");
    report.nonexistent = result;

    let url = &config.cleartext_url;
    eprint!("Check cleartext resource ({url}) ... ");
    let result = check_content(&custom_client, url, &config.content);
    eprintln!("{result}");
    report.cleartext = result;

    let url = &config.target_url;
    eprint!("Check target resource ({url}) ... ");
    let result = check_content(&system_client, url, &config.content);
    eprintln!("{result}");
    report.target = result;

    Ok(report)
}

fn custom_dns_client(config: &TargetConfig) -> DNSClient {
    let servers = config
        .bootstrap_dns
        .iter()
        .map(|i| UpstreamServer::new(SocketAddr::new(*i, 53)))
        .collect();

    DNSClient::new(servers)
}

fn format_random_domain(template: &str) -> String {
    let chars = Alphanumeric.sample_string(&mut rand::rng(), 16);
    template.replace("{random}", &chars)
}

fn check_nonexistent(client: &Agent, url: &str) -> TestResult {
    let url = format_random_domain(url);

    match client.get(&url).call() {
        Ok(_response) => TestResult::Fail("unexpected response".to_string()),
        Err(error) => match error {
            ureq::Error::HostNotFound => TestResult::Pass,

            _ => TestResult::Error(Box::new(error)),
        },
    }
}

fn check_content(client: &Agent, url: &str, expected_content: &str) -> TestResult {
    match client.get(url).call() {
        Ok(mut response) => {
            if response.status() != 200 {
                return TestResult::Fail(format!("unexpected status code {}", response.status()));
            }

            let content = response.body_mut().read_to_vec().unwrap_or_default();

            if content != expected_content.as_bytes() {
                let mut snippet = content.escape_ascii().to_string();
                snippet.truncate(64);

                TestResult::Fail(format!("unexpected content '{snippet}'",))
            } else {
                TestResult::Pass
            }
        }
        Err(error) => TestResult::Error(Box::new(error)),
    }
}
