use std::net::IpAddr;
use tokio;

use ipnet::IpNet;
use ipnet::Ipv4Net;

use core::net::Ipv6Addr;
use std::ops::BitAnd;
use std::ops::BitAndAssign;

use rtnetlink::{new_connection, Error as rtnetlinkErr, Handle};

mod db;
mod interface;
mod net;

use crate::db::clickhouse;
use crate::interface::info;
use crate::net::address;

#[tokio::main]
async fn main() -> Result<(), rtnetlinkErr> {
    let con = clickhouse::connect().await;

    let get_stat = r"
        SELECT * FROM stat;
    ";

    let res = con.execute(get_stat).await;

    match res {
        Ok(res) => println!("{:?}", res),
        Err(err) => panic!("Select error: {:?}", err),
    }

    // Connection to a Netlink socket
    let connect = new_connection();
    let handle: Handle;

    match connect {
        Ok((connection, get_handle, _)) => {
            handle = get_handle;
            // Running in the background (asynchronously)
            tokio::spawn(connection);
        }
        Err(_) => panic!("Connection failed"),
    }

    let addr = Ipv6Addr::new(
        0x1020, 0x3040, 0x5060, 0x7080, 0x90A0, 0xB0C0, 0xD0E0, 0xF00D,
    );

    let mask: u128 = !0 << (128 - 64);
    let bitAddr = u128::from(addr);

    let result: u128 = bitAddr & mask;
    let network = Ipv6Addr::from(result);

    //println!("{:?}", network);

    //let g = info::get_all_ptp_interfaces(&handle).await?;
    //let d = info::get_all_ptp_interfaces(&handle).await?;
    //let d = info::get_all_ptp_interfaces(&handle).await?;
    //let point_to_point = info::is_point_to_point(&handle, 2).await;

    // if let Err(err) = point_to_point {
    //     info::err_netlink_info(err);
    // }
    //let interface_address = info::get_all_interfaces(&handle).await?;
    //let interface_addr = info::get_interface_address(&handle, 8).await?;
    let interface_addr = info::get_interface_stats(&handle, 2).await?;

    //address::get_network_addr(&interface_addr[2]);
    println!("{:#?}", interface_addr);

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
