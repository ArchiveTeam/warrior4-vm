use network_interface::NetworkInterfaceConfig;

pub fn get_eth0_ip_address() -> String {
    let mut ip_addresses = Vec::new();
    let interfaces = network_interface::NetworkInterface::show().unwrap_or_default();

    for interface in interfaces {
        if interface.name == "eth0" {
            for addr in interface.addr {
                ip_addresses.push(addr.ip().to_string());
            }
        }
    }

    ip_addresses.join(", ")
}
