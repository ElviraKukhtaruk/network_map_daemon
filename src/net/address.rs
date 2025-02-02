use ipnet::IpNet;
use ipnet::Ipv4Net;
use std::net::IpAddr;

use crate::interface::info::interface;

pub fn get_network_addr6(interface_addr: &interface::IntterfaceAddr) {
    let addr = interface_addr.address.to_string();
    let prefix = interface_addr.prefix_len.to_string();

    let addr_cidr = format!("{addr}/{prefix}");

    println!("{:?}", addr_cidr);

    let net: IpNet = addr_cidr.parse().unwrap();

    println!("{:#?}", net.network());
}
