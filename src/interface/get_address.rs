use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
use clickhouse::Client;
use regex::Regex;
use rtnetlink::Handle;
use log::{error, info, warn};
use tokio::time::interval;

use crate::db::queries::{add_addr, delete_addr, delete_data_efficiently, get_addr, update_addr};
use crate::{config::config::ServerConfiguration, db::schema::Addr};
use crate::interface::info;
use super::info::{get_all_interfaces, get_interface_name_from_attribute, get_loopback_from_header};

#[derive(Debug)]
pub struct Updates {
    pub updates: Vec<Addr>,
    pub deletes: Vec<Addr>,
    pub creates: Vec<Addr>,
}

pub async fn get_interface_addresses(handle: &Handle, rules: &[Option<String>], config: &ServerConfiguration, verbose: bool) -> Result<Vec<Addr>, rtnetlink::Error> {

    let compiled_rules: Vec<Option<Regex>> = rules
        .iter()
        .map(|rule_opt| rule_opt.as_ref().and_then(|pattern| Regex::new(pattern).ok()))
        .collect();
    let compiled_rules = Arc::new(compiled_rules);

    // Get all network interfaces.
    let interfaces = get_all_interfaces(handle).await?;

    // Process each interface concurrently with a limit of 10.
    let max_concurrent = 10;
    let addrs: Vec<Addr> = futures::stream::iter(interfaces)
        .map(|interface| {
            let compiled_rules = Arc::clone(&compiled_rules); // Clone the Arc for each closure
            async move {
                let int_name = get_interface_name_from_attribute(interface.attributes);
                let is_loopback = get_loopback_from_header(interface.header);

                if let Some(name) = int_name {
                    let interface_name_match = compiled_rules.iter().any(|opt_regex| {
                        if let Some(regex) = opt_regex {
                            regex.is_match(&name)
                        } else {
                            // If rule is None, allow non-loopback interfaces.
                            !is_loopback
                        }
                    });

                    if interface_name_match && !is_loopback {
                        get_addresses(handle, name, config, verbose).await.ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        })
        .buffer_unordered(max_concurrent)
        .filter_map(|opt| async move { opt })
        .collect()
        .await;
    Ok(addrs)
}

pub async fn get_addresses(handle: &Handle, name: String, config: &ServerConfiguration, verbose: bool) -> Result<Addr, rtnetlink::Error> {
    let interface_addr = info::get_interface_address(&handle, &name).await?;
    let mut addresses: Vec<(Option<Ipv6Addr>, Option<u8>)> = Vec::new();
    let mut peers: Vec<(Option<Ipv6Addr>, Option<u8>)> = Vec::new();

    for addr in &interface_addr {
        let ip_addr = addr.address;
        let ip_local = addr.local;

        // Push to addresses: prefer ip_local, then ip_addr, else (None, None)
        let to_push_addresses = if ip_local.0.is_some() {
            ip_local
        } else if ip_addr.0.is_some() {
            ip_addr
        } else {
            (None, None)
        };
        addresses.push(to_push_addresses);

        // Push to peers: when both present and different, or both absent
        if ip_local.0.is_some() && ip_addr.0.is_some() && ip_local != ip_addr {
            peers.push(ip_addr);
        } else if ip_local.0.is_none() && ip_addr.0.is_none() {
            peers.push((None, None));
        }

        if verbose {
            match (ip_local.0, ip_addr.0) {
                (Some(local), Some(addr)) if ip_local != ip_addr => {
                    info!("{name}: Peer address {}, Local address {}", addr, local);
                }
                (Some(local), _) => {
                    info!("{name}: Address {}", local);
                }
                (None, Some(addr)) => {
                    info!("{name}: Address {}", addr);
                }
                (None, None) => {
                    warn!("Interface '{name}' has no addresses.");
                }
            }
        }
    }

    Ok(Addr {
        server_id: config.get_config().server_id.to_string(),
        interface: name,
        ipv6: addresses,
        ipv6_peer: peers
    })
}

pub async fn add_addr_to_database(handle: &Handle, client: &Client, server: &ServerConfiguration) {

    info!("Adding interfaces' IPv6/IPv4-mapped addresses...");

    // Delete data efficiently
    if let Err(e) = delete_data_efficiently(client, &server.get_config().server_id).await {
        error!("Failed to delete addresses from the database: {e}. Exiting...");
        process::exit(1);
    }

    // Get interface addresses
    let addrs = match get_interface_addresses(&handle, &server.get_config().interface_filter, &server, true).await {
        Ok(addrs) => addrs,
        Err(e) => {
            error!("An error occurred while getting addresses: {e}. Exiting...");
            process::exit(1);
        }
    };

    // Add addresses to the database
    add_addr(client, addrs).await.inspect_err(|e| {
        error!("Failed to add addresses to the database: {e}. Exiting...");
        process::exit(1);
    }).ok();
}

pub async fn check_for_interface_updates(handle: &Handle, client: &Client, server: &ServerConfiguration) {
    let mut interval = interval(Duration::from_secs(5));
    info!("Cheking for interface updates [5 seconds].");


    loop {
        interval.tick().await;
        info!("Checking for interfaces updates...");

        // Get interface addresses concurrently.
        let addresses = get_interface_addresses(&handle, &server.get_config().interface_filter, &server, false).await;
        let db_addrs = get_addr(&client, &server.get_config()).await;

        if let (Ok(addrs), Ok(db_addr)) = (addresses, db_addrs) {

            let diff = compare(&addrs, &db_addr);

            if !diff.creates.is_empty() {
                info!("Creating new interfaces (Update)");
                add_addr(client, diff.creates).await.ok();
            }

            if !diff.updates.is_empty() {
                info!("Updating interfaces (Update)");
                update_addr(client, diff.updates).await.ok();
            }

            if !diff.deletes.is_empty() {
                info!("Deleting interfaces (Update)");
                delete_addr(client, diff.deletes).await.ok();
            }
        }
    }
}

pub fn compare(fresh: &Vec<Addr>, db: &Vec<Addr>) -> Updates {
    let mut updates = Vec::new();
    let mut creates = Vec::new();
    let mut deletes = Vec::new();

    // Build lookup maps from interface name to Addr.
    let fresh_map: HashMap<&String, &Addr> = fresh.iter().map(|addr| (&addr.interface, addr)).collect();
    let db_map: HashMap<&String, &Addr> = db.iter().map(|addr| (&addr.interface, addr)).collect();

    // For each interface in the fresh (local) data:
    // - If the interface exists in the database, but its details differ, mark it for update.
    // - If the interface is new (not found in the database), mark it for creation.
    for (iface, fresh_addr) in &fresh_map {
        if let Some(db_addr) = db_map.get(iface) {
            if fresh_addr.ipv6 != db_addr.ipv6 || fresh_addr.ipv6_peer != db_addr.ipv6_peer {
                updates.push((*fresh_addr).clone());
            }
        } else {
            creates.push((*fresh_addr).clone());
        }
    }

    // For interfaces in the database that are missing in the fresh data, mark them for deletion.
    for (iface, delete) in &db_map {
        if !fresh_map.contains_key(iface) {
            deletes.push((*delete).clone());
        }
    }

    Updates { updates, creates, deletes }
}
