use ipnet::IpNet;
use std::{net::IpAddr, str::FromStr};

pub fn get_network_addr6(interface_addr: &IpAddr, address_prefix: u8) -> Result<IpNet, <IpNet as FromStr>::Err> {
    let addr = interface_addr.to_string();
    let prefix = address_prefix.to_string();

    let addr_cidr = format!("{addr}/{prefix}");

    println!("{:?}", addr_cidr);

    let net: IpNet = addr_cidr.parse()?;

    println!("{:#?}", net.network());

    return Ok(net);
}
