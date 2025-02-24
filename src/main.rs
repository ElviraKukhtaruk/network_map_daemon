use std::net::IpAddr;
use std::net::SocketAddr;
use interface::addr::get_addresses;
use netlink_packet_route::link::LinkAttribute;
use netlink_packet_route::RouteNetlinkMessage;
use tokio;

use ipnet::IpNet;
use ipnet::Ipv4Net;

use core::net::Ipv6Addr;
use std::ops::BitAnd;
use std::ops::BitAndAssign;

use rtnetlink::{new_connection, Error as rtnetlinkErr, Handle};
use futures_channel::mpsc::UnboundedReceiver;
use netlink_packet_core::{ NetlinkMessage };
use netlink_sys;
use futures::stream::StreamExt;

mod db;
mod interface;
mod net;
mod errors;
mod config;

use crate::config::config:: { DbConnection, ServerConfiguration };
use crate::db::queries;

use crate::db::schema;
use crate::interface::{ info, addr };

#[tokio::main]
async fn main() -> Result<(), rtnetlinkErr> {

    let con = DbConnection::new().await;
    let client = con.get_client();

    let a = ServerConfiguration::new(con.get_config());

     println!("{:?}", a.get_config().server);


    // ServerConfig::new();
    //let server = queries::add_addr(client).await;


    //queries::add_interface(client).await;


    // Connection to a Netlink socket
    let connect = new_connection();
    let handle: Handle;

    match connect {
        Ok((connection, get_handle, _)) => {
            handle = get_handle;
            // Running in the background (asynchronously)
            tokio::spawn(connection);
        }
        Err(_) => panic!("RTNetLink Connection failed"),
    }



    //println!("{:?}", messages);

    let addr = Ipv6Addr::new(
        0x1020, 0x3040, 0x5060, 0x7080, 0x90A0, 0xB0C0, 0xD0E0, 0xF00D,
    );

    let mask: u128 = !0 << (128 - 64);
    let bitAddr = u128::from(addr);

    let result: u128 = bitAddr & mask;
    let network = Ipv6Addr::from(result);



//    let addrs = get_addresses(&handle, 2).await?;

  //  println!("{:?}", addrs);

    //let interface_addr = info::get_interface_stats(&handle, 2).await?;

    //address::get_network_addr(&interface_addr[2]);
    /*for attr in &interface_addr[0].attributes {
        match attr {
            LinkAttribute::IfName(name) => println!("{:?}", name),
            _ => ()
        }
        println!("{:?}", interface_addr);
    }*/

    //let addr1 = interface_addr[0].address.as_str();
    //let pref1 = interface_addr[0].prefix_len.as_str();

    //let addr_cidr = format!("{addr1}/{pref1}");

    //println!("{:?}", addr_cidr);

    /*match interface_addr[0].address {
        IpAddr::V4(ipv4) => {
            let addr_str = ipv4.to_string();
            let prefix_str = interface_addr[0].prefix_len.to_string();

            let addr_cidr = format!("{addr_str}/{prefix_str}");

            println!("{:?}", ipv4.to_string());
            println!("{:?}", addr_cidr);
        },
        IpAddr::V6(ipv6) => {
            println!("{:?}", ipv6.to_string());
        }
    }*/

    Ok(())
}
