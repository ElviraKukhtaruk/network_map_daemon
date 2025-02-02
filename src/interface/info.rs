use std::{num::NonZeroI32, net::{IpAddr}};
use rtnetlink::{Error as rtnetlinkErr, Handle};

use syscalls::{Errno as syscallErr};

use netlink_packet_route::{link::{LinkMessage, LinkAttribute, LinkFlag}};
use netlink_packet_route::{address::{AddressMessage, AddressAttribute::{Address, Local}}};

use futures_util::TryStreamExt;

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
                rx_bytes: status.rx_bytes
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

pub async fn get_interface_address(handle: &Handle, index: u32) -> Result<Vec<interface::IntterfaceAddr>, rtnetlinkErr> {
    let address_handle = handle.address().get();
    let get_address = address_handle.set_link_index_filter(index);

    let result_response_address: Result<Vec<AddressMessage>, rtnetlinkErr> = get_address.execute().try_collect().await;
    let response_address: Vec<AddressMessage> = result_response_address.into_iter().flatten().collect();

    let mut peers: Vec<interface::IntterfaceAddr> = vec![];

    println!("{:#?}", response_address);
    for address_message in &response_address {
        let address_attributes = &address_message.attributes;
        let mut peering: Vec<IpAddr> = Vec::with_capacity(2);

        address_attributes.into_iter().for_each(|e| {
           
            match e {
                // Address will be always first
                Address(addr_peer) => peering.insert(0, *addr_peer),
                Local(addr_local) => peering.push(*addr_local),
                _ => () 
            }
        });

        if peering.len() == 2 {
            let peer = interface::IntterfaceAddr { address: peering[0], local: peering[1], prefix_len: address_message.header.prefix_len };
            peers.push(peer);
        } else if peering.len() == 1 {
            let peer = interface::IntterfaceAddr { address: peering[0], local: peering[0], prefix_len: address_message.header.prefix_len };
            peers.push(peer);
        } else {
            return Err(rtnetlinkErr::RequestFailed);
        }
    }
    if response_address.len() > 0 { Ok(peers) }
    else { Err(rtnetlinkErr::RequestFailed) }
}