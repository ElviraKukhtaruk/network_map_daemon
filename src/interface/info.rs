use std::net::IpAddr;
use rtnetlink::{Error as rtnetlinkErr, Handle};
use klickhouse::Ipv6;
use netlink_packet_route::link::{ LinkAttribute, LinkFlag, LinkHeader, LinkMessage };
use netlink_packet_route::address::{ AddressMessage, AddressAttribute::{ Address, Local }};

use futures_util::TryStreamExt;

pub mod interface;

pub async fn get_all_interfaces(handle: &Handle) -> Result<Vec<LinkMessage>, rtnetlinkErr> {
    let link_handle = handle.link().get();
    let stream = link_handle.execute();

    let response_link = stream.try_collect::<Vec<LinkMessage>>().await?;
    Ok(response_link)
}

pub async fn get_interface_status(handle: &Handle, name: &String) -> Result<bool, rtnetlinkErr> {
    let response_link = get_interface(&handle, name).await?;
    for header in response_link.header.flags.iter() {
        if let LinkFlag::Up = header {
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn get_interface_stats(handle: &Handle, name: &String) -> Result<interface::Stats, rtnetlinkErr> {
    let response_link = get_interface(&handle, name).await?;
    let mut int_name = String::from("");

    for attribute in response_link.attributes.iter() {
        if let LinkAttribute::IfName(interface) = attribute { int_name = interface.to_string(); }

        if let LinkAttribute::Stats64(status) = attribute {
            return Ok(interface::Stats {
                int_name,
                tx_packets: status.tx_packets,
                rx_packets: status.rx_packets,
                tx_bytes: status.tx_bytes,
                rx_bytes: status.rx_bytes,
                tx_dropped: status.tx_dropped,
                rx_dropped: status.rx_dropped
            });
        }
    }
    Err(rtnetlinkErr::RequestFailed)
}

pub async fn get_interface(handle: &Handle, name: &String) -> Result<LinkMessage, rtnetlinkErr> {
    let link_handle = handle.link().get();
    let get_link = link_handle.match_name(name.clone());
    let response_link = get_link.execute().try_next().await?;

    if let Some(link) = response_link { return Ok(link); }
    Err(rtnetlinkErr::RequestFailed)
}

pub async fn get_index_by_name(handle: &Handle, name: &String) -> Result<u32, rtnetlinkErr> {
    let response_link = get_interface(&handle, name).await?;

    Ok(response_link.header.index)
}

pub fn get_interface_name_from_attribute(attr: Vec<LinkAttribute>) -> Option<String> {
    for attribute in attr.iter() {
        if let LinkAttribute::IfName(interface) = attribute { return Some(interface.to_string()); }
    }
    None
}

pub fn get_loopback_from_header(header: LinkHeader) -> bool {
    return header.flags.into_iter().any(|e| match e {
        LinkFlag::Loopback => true,
        _ => false
    });
}

pub async fn get_interface_address(handle: &Handle, name: &String) -> Result<Vec<interface::InterfaceAddr>, rtnetlinkErr> {
    let address_handle = handle.address().get();
    let interface_index = get_index_by_name(handle, name).await?;
    let get_address = address_handle.set_link_index_filter(interface_index);

    let result_response_address: Result<Vec<AddressMessage>, rtnetlinkErr> = get_address.execute().try_collect().await;
    let response_address: Vec<AddressMessage> = result_response_address.into_iter().flatten().collect();

    let mut int_addresses: Vec<interface::InterfaceAddr> = vec![];

    for address_message in &response_address {
        let address_attributes = &address_message.attributes;

        let mut address: Option<IpAddr> = None;
        let mut local: Option<IpAddr> = None;

        let mut address_mapped: Option<Ipv6> = None;
        let mut local_mapped: Option<Ipv6> = None;

        address_attributes.into_iter().for_each(|e| {
            match e {
                Address(addr_peer) => address = Some(*addr_peer),
                Local(addr_local) => local = Some(*addr_local),
                _ => ()
            }
        });

        match address {
            Some(IpAddr::V4(v4)) => address_mapped = Some(Ipv6(v4.to_ipv6_mapped())),
            Some(IpAddr::V6(v6)) => address_mapped = Some(Ipv6(v6)),
            None => ()
        }

        match local {
            Some(IpAddr::V4(v4)) => local_mapped = Some(Ipv6(v4.to_ipv6_mapped())),
            Some(IpAddr::V6(v6)) => local_mapped = Some(Ipv6(v6)),
            None => ()
        }

        let int_addr = interface::InterfaceAddr {
            address: (address_mapped, Some(address_message.header.prefix_len)),
            local: (local_mapped, Some(address_message.header.prefix_len))
        };

        int_addresses.push(int_addr);
    }
    if response_address.len() > 0 || response_address.is_empty() {
        Ok(int_addresses)
    } else {
        Err(rtnetlinkErr::RequestFailed)
    }
}
