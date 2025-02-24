use rtnetlink::Handle;
use klickhouse:: Ipv6;
use crate::db::schema::Addr;

use crate::interface::info;

pub async fn get_addresses(handle: &Handle, index: u32) -> Result<Addr, rtnetlink::Error>{
    let interface_addr = info::get_interface_address(&handle, index).await?;
    let mut addresses: Vec<(Option<Ipv6>, Option<u8>)> = Vec::new();
    let mut peers: Vec<(Option<Ipv6>, Option<u8>)> = Vec::new();

    for addr in &interface_addr {
        let ip_addr: (Option<Ipv6>, Option<u8>) = addr.address;
        let ip_local: (Option<Ipv6>, Option<u8>) = addr.local;

        if ip_addr.0.is_some() && ip_local.0.is_some() && ip_addr != ip_local {
            // Store local address and peer's address
            peers.push(ip_addr);
            addresses.push(ip_local);
        } else if ip_addr.0.is_some() && ip_local.0.is_some() && ip_addr == ip_local {
            // Both address and local are equal (Store only one of them)
            addresses.push(ip_addr);
        } else if ip_addr.0.is_none() && ip_local.0.is_some() {
            addresses.push(ip_local);
        } else if ip_addr.0.is_some() && ip_local.0.is_none() {
            addresses.push(ip_addr);
        } else {
            // Without addresses
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
