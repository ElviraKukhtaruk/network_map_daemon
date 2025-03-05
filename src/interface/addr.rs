use rtnetlink::Handle;
use klickhouse:: Ipv6;
use crate::db::schema::Addr;
use log::{info, warn};

use crate::interface::info;

pub async fn get_addresses(handle: &Handle, name: String) -> Result<Addr, rtnetlink::Error> {
    let interface_addr = info::get_interface_address(&handle, &name).await?;
    let mut addresses: Vec<(Option<Ipv6>, Option<u8>)> = Vec::new();
    let mut peers: Vec<(Option<Ipv6>, Option<u8>)> = Vec::new();

    info!("Adding interfaces' IPv6/IPv4-mapped addresses...");
    for addr in &interface_addr {
        let ip_addr: (Option<Ipv6>, Option<u8>) = addr.address;
        let ip_local: (Option<Ipv6>, Option<u8>) = addr.local;

        let print_addr = ip_addr.0.map_or_else(|| "None".to_string(), |addr| addr.to_string());
        let print_local = ip_local.0.map_or_else(|| "None".to_string(), |addr| addr.to_string());

        if ip_addr.0.is_some() && ip_local.0.is_some() && ip_addr != ip_local {
            // Store local address and peer's address
            info!("{name}: Peer address {print_addr}, Local address {print_local}");
            peers.push(ip_addr);
            addresses.push(ip_local);
        } else if ip_addr.0.is_some() && ip_local.0.is_some() && ip_addr == ip_local {
            // Both address and local are equal (Store only one of them)
            info!("{name}: Address {print_addr}");
            addresses.push(ip_addr);
        } else if ip_addr.0.is_none() && ip_local.0.is_some() {
            info!("{name}: Address address {print_local}");
            addresses.push(ip_local);
        } else if ip_addr.0.is_some() && ip_local.0.is_none() {
            info!("{name}: Address address {print_addr}");
            addresses.push(ip_addr);
        } else {
            // Without addresses
            warn!("Interface '{name}' has no addresses.");
            addresses.push((None, None));
            peers.push((None, None));
        }
    }

    Ok(Addr {
        server_id: "Haha".to_string(),
        interface: "Int".to_string(),
        ipv6: addresses,
        ipv6_peer: peers
    })
}
