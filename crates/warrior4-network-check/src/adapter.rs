use std::{net::SocketAddr, sync::Arc};

use hickory_resolver::TokioResolver;
use reqwest::dns::{Addrs, Resolve};

/// Adapter for reqwest to hickory-resolver.
#[derive(Debug)]
pub struct DnsClientAdapter {
    client: Arc<TokioResolver>,
}

impl DnsClientAdapter {
    pub fn new(client: TokioResolver) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

impl Resolve for DnsClientAdapter {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let client = self.client.clone();
        Box::pin(async move {
            let ip_result = client.lookup_ip(name.as_str()).await?;
            Ok(Box::new(
                ip_result
                    .into_iter()
                    .map(|ip_addr| SocketAddr::new(ip_addr, 0)),
            ) as Addrs)
        })
    }
}
