use std::{
    net::{Ipv4Addr, SocketAddr},
    ops::Deref,
    sync::RwLock,
};

use dnsclient::sync::DNSClient;
use ureq::unversioned::resolver::{ResolvedSocketAddrs, Resolver};
use url::Url;

/// Adapter for dnsclient to ureq.
#[derive(Debug)]
pub struct DnsClientAdapter {
    client: RwLock<DNSClient>,
}

impl DnsClientAdapter {
    pub fn new(client: DNSClient) -> Self {
        Self {
            client: RwLock::new(client),
        }
    }
}

impl Resolver for DnsClientAdapter {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        _config: &ureq::config::Config,
        timeout: ureq::unversioned::transport::NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let mut client = self.client.write().unwrap();
        client.set_timeout(timeout.after.deref().to_owned());

        let url =
            Url::parse(&uri.to_string()).map_err(|error| ureq::Error::BadUri(error.to_string()))?;

        let host = url
            .host_str()
            .ok_or_else(|| ureq::Error::BadUri("host".to_string()))?;
        let port = url
            .port_or_known_default()
            .ok_or_else(|| ureq::Error::BadUri("port".to_string()))?;

        let addresses = client.query_addrs(host)?;

        if addresses.is_empty() {
            return Err(ureq::Error::HostNotFound);
        }

        let mut socket_addresses =
            ResolvedSocketAddrs::from_fn(|_i| SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0));

        for address in addresses.iter().take(16) {
            socket_addresses.push(SocketAddr::new(*address, port));
        }

        assert!(!socket_addresses.is_empty());

        Ok(socket_addresses)
    }
}
