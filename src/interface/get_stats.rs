use log::{info, error, warn};
use klickhouse::{Client, DateTime};
use rtnetlink::{Error, Handle};
use regex::Regex;
use tokio::time::{interval, Duration};
use crate::config::config::ServerConfiguration;
use crate::queries::add_stat;
use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
use crate::db::schema::Stat;
use super::info::{get_all_interfaces, get_interface_name_from_attribute, get_interface_stats, get_loopback_from_header};

pub async fn get_stats(handle: &Handle, name: &String, config: &ServerConfiguration) -> Option<Stat> {
    let stats = get_interface_stats(handle, name).await.inspect_err(|err| error!("Failed to get stats: {}", err));

    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).inspect_err(|err| error!("Failed to get timestamp: {}", err));

    if let (Ok(stat), Ok(timestamp)) = (stats, since_the_epoch) {
        let server_id = &config.get_config().server_id.as_str();

        return Some(Stat {
            server_id: server_id.to_string(),
            interface: stat.int_name,
            timestamp: DateTime(klickhouse::Tz::UTC, timestamp.as_secs() as u32),
            rx: stat.rx_bytes,
            tx: stat.tx_bytes,
            rx_p: stat.rx_packets,
            tx_p: stat.tx_packets,
            rx_d: stat.rx_dropped,
            tx_d: stat.tx_dropped
        });
    }
    None
}


pub async fn filter_interfaces(
    handle: &Handle,
    client: &Client,
    rules: &Vec<Option<String>>,
    config: &ServerConfiguration,
    last_data: &mut HashMap<std::string::String, std::option::Option<Stat>>,
) -> Result<(), Error> {

    let interfaces = get_all_interfaces(handle).await?;

    for interface in interfaces {
        let int_name = get_interface_name_from_attribute(interface.attributes);
        let is_loopback = get_loopback_from_header(interface.header);

        if let Some(name) = int_name.as_ref() {
            // Match interface name with regex string
            let interface_name_match = rules.iter().any(|s| {
                s.as_ref().map_or_else(|| {
                    if !is_loopback {
                        return true;
                    }
                    false
                }, |result| {
                    Regex::new(result.as_str()).map_or_else(|err| {
                        error!("Error occurred while creating regex for pattern '{result}': {}", err);
                        false
                    }, |regex| regex.is_match(name))
                })
            });

            if interface_name_match && !is_loopback {
                let stats = get_stats(handle, name.into(), config).await;
                save_stat(client, last_data, stats).await;
            } else if rules.len() == 0 && !is_loopback {
                let stats = get_stats(handle, name.into(), config).await;
                save_stat(client, last_data, stats).await;
            }
        }
    }
    Ok(())
}

pub async fn save_stat(
    client: &Client,
    last_data: &mut HashMap<std::string::String, std::option::Option<Stat>>,
    stats: Option<Stat> ) {

        match stats {
            Some(curr_stat) => {

                let server_id = curr_stat.server_id.as_str();
                let interface = curr_stat.interface.as_str();

                if let Some(Some(old_data)) = &last_data.get(&curr_stat.interface) {
                    let old_time = old_data.timestamp.1;
                    let dt = (curr_stat.timestamp.1 - old_time) as u64;

                    println!("[{interface}] {:?} - {:?} = {:?}", curr_stat.tx, old_data.tx, (curr_stat.tx - old_data.tx));
                    add_stat(client, Stat {
                        server_id: server_id.into(),
                        interface: interface.into(),
                        timestamp: curr_stat.timestamp,
                        tx_p: (curr_stat.tx_p - old_data.tx_p) / dt,
                        rx_p: (curr_stat.rx_p - old_data.rx_p) / dt,
                        tx: ((curr_stat.tx - old_data.tx) * 8) / dt,
                        rx: ((curr_stat.rx - old_data.rx) * 8) / dt,
                        tx_d: (curr_stat.tx_d - old_data.tx_d) / dt,
                        rx_d: (curr_stat.rx_d - old_data.rx_d) / dt
                    }).await.ok();
                }

                last_data.insert(interface.into(), Some(Stat {
                    server_id: curr_stat.server_id,
                    interface: interface.into(),
                    timestamp: curr_stat.timestamp,
                    tx_p: curr_stat.tx_p,
                    rx_p: curr_stat.rx_p,
                    tx: curr_stat.tx,
                    rx: curr_stat.rx,
                    tx_d: curr_stat.tx_d,
                    rx_d: curr_stat.rx_d
                }));

            },
            None => ()
        }



}

pub async fn save_stats_every_second(handle: &Handle, server_config: &ServerConfiguration, client: &Client) {
    let mut interval = interval(Duration::from_secs(1));
    let mut last_stats: HashMap<String, Option<Stat>> = HashMap::new();

    loop {
        interval.tick().await;

        filter_interfaces(
            &handle,
            &client,
            &server_config.get_config().interface_filter,
            server_config,
            &mut last_stats
        ).await.inspect_err(|err| error!("Failed to save interfaces stats: {}", err)).ok();

    }
}
