use std::{num::NonZeroI32, net::{ IpAddr, Ipv4Addr, Ipv6Addr }};
use rtnetlink::{Error as rtnetlinkErr, Handle};
use klickhouse::Ipv6;

use syscalls::{Errno as syscallErr};

use netlink_packet_route::{address::AddressAttribute, link::{LinkAttribute, LinkFlag, LinkMessage}};
use netlink_packet_route::{address::{AddressMessage, AddressAttribute::{Address, Local, Label}}};

use futures_util::TryStreamExt;
use tokio::time::interval;

pub mod interface;

pub async fn get_all_interfaces(handle: &Handle) -> Result<Vec<LinkMessage>, rtnetlinkErr> {
    let link_handle = handle.link().get();

    let stream = link_handle.execute();

    let response_link = stream.try_collect::<Vec<LinkMessage>>().await?;

    Ok(response_link)
}

pub async fn get_all_ptp_interfaces(handle: &Handle) -> Result<Vec<LinkMessage>, rtnetlinkErr> {
    let response_link = get_all_interfaces(handle).await?;

    let response_link_pointopoint: Vec<_> = response_link.clone().into_iter().filter(|e| {
        return e.header.flags.clone().into_iter().any(|flag| match flag {
            LinkFlag::Pointopoint => true,
            _ => false
        });
    }).collect();

    Ok(response_link_pointopoint)
}

pub async fn is_point_to_point(handle: &Handle, index: u32) -> Result<bool, rtnetlinkErr> {
    let response_link = get_interface(&handle, index).await?;

    return Ok(response_link.header.flags.into_iter().any(|e| match e {
        LinkFlag::Pointopoint => true,
        _ => false
    }));
}

pub async fn get_interface_status(handle: &Handle, index: u32) -> Result<bool, rtnetlinkErr> {
    let response_link = get_interface(&handle, index).await?;
    for header in response_link.header.flags.iter() {
        if let LinkFlag::Up = header {
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn get_interface_stats(handle: &Handle, index: u32) -> Result<interface::Stats, rtnetlinkErr> {
    let response_link = get_interface(&handle, index).await?;
    let int_status = get_interface_status(&handle, index).await?;
    let mut int_name = String::from("");

    for attribute in response_link.attributes.iter() {
        if let LinkAttribute::IfName(interface) = attribute { int_name = interface.to_string(); }

        if let LinkAttribute::Stats64(status) = attribute {
            return Ok(interface::Stats {
                int_status,
                int_name: int_name,
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

pub async fn get_interface(handle: &Handle, index: u32) -> Result<LinkMessage, rtnetlinkErr> {
    let link_handle = handle.link().get();
    let get_link = link_handle.match_index(index);
    let response_link = get_link.execute().try_next().await?;

    if let Some(link) = response_link { return Ok(link); }
    Err(rtnetlinkErr::RequestFailed)
}

pub fn err_netlink_info(error: rtnetlinkErr){
    match error {
        rtnetlinkErr::NetlinkError(err_message) => {
            if let Some(code) = err_message.code {
                let code: i32 = NonZeroI32::abs(code).get();
                eprintln!("{:?}", syscallErr::name_and_description(&syscallErr::new(code)));
            }
        },
        _ => ()
    }
}

pub async fn get_interface_address(handle: &Handle, index: u32) -> Result<Vec<interface::InterfaceAddr>, rtnetlinkErr> {
    let address_handle = handle.address().get();
    let get_address = address_handle.set_link_index_filter(index);

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
