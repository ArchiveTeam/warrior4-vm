use std::{net::IpAddr, sync::Arc};

use hickory_resolver::{
    TokioResolver,
    config::{NameServerConfig, ResolveHosts, ResolverConfig, ResolverOpts},
    net::runtime::TokioRuntimeProvider,
};

use crate::config::TargetConfig;

pub fn build_custom_public_dns_resolver(config: &TargetConfig) -> TokioResolver {
    build_udp_resolver(&config.public_dns_resolver)
}

pub fn build_custom_cleartext_dns_resolver(config: &TargetConfig) -> TokioResolver {
    build_udp_resolver(&config.cleartext_dns_resolver)
}

fn build_udp_resolver(addresses: &[IpAddr]) -> TokioResolver {
    let mut nameserver_configs = Vec::new();

    for address in addresses {
        nameserver_configs.push(NameServerConfig::udp(*address));
    }

    let resolver_config = ResolverConfig::from_name_servers(nameserver_configs);

    let mut builder =
        TokioResolver::builder_with_config(resolver_config, TokioRuntimeProvider::default());
    config_resolver(builder.options_mut());

    builder.build().unwrap()
}

fn config_resolver(options: &mut ResolverOpts) {
    options.use_hosts_file = ResolveHosts::Never;
    options.attempts = 4;
    // Quad9 does not seem to like multiple TLS connections.
    options.num_concurrent_reqs = 1;
}

pub fn build_custom_encrypted_dns_resolver(config: &TargetConfig) -> TokioResolver {
    let mut nameserver_configs = Vec::new();

    for (address, hostname) in &config.encrypted_dns_resolver {
        nameserver_configs.push(NameServerConfig::tls(
            *address,
            Arc::from(hostname.as_str()),
        ));
    }

    let resolver_config = ResolverConfig::from_name_servers(nameserver_configs);

    let mut builder =
        TokioResolver::builder_with_config(resolver_config, TokioRuntimeProvider::default());
    config_resolver(builder.options_mut());

    builder.build().unwrap()
}

pub fn build_system_dns_resolver() -> TokioResolver {
    let builder = TokioResolver::builder_tokio().unwrap();

    builder.build().unwrap()
}
