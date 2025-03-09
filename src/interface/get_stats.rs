use log::{error, info};
use clickhouse::Client;
use rtnetlink::Handle;
use regex::Regex;
use tokio::time::{interval, Duration};
use crate::config::config::ServerConfiguration;
use crate::queries::add_stat;
use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
use crate::db::schema::Stat;
use super::info::{get_all_interfaces, get_interface_name_from_attribute, get_interface_stats, get_loopback_from_header};
use futures::stream::StreamExt;
use std::sync::Arc;

pub async fn get_stats(handle: &Handle, name: &String, config: &ServerConfiguration) -> Option<Stat> {
    let stats = get_interface_stats(handle, name).await.inspect_err(|err| error!("Failed to get stats: {}", err));

    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).inspect_err(|err| error!("Failed to get timestamp: {}", err));

    if let (Ok(stat), Ok(timestamp)) = (stats, since_the_epoch) {
        let server_id = &config.get_config().server_id.as_str();

        return Some(Stat {
            server_id: server_id.to_string(),
            interface: stat.int_name,
            timestamp: timestamp.as_secs() as u32,
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

pub async fn save_stats_every_second(handle: &Handle, server_config: &ServerConfiguration, client: &Client) {
    let mut interval = interval(Duration::from_secs(1));
    // For concurrent updates.
    let last_stats = Arc::new(tokio::sync::Mutex::new(HashMap::<String, Option<Stat>>::new()));
    info!("Collecting and saving statistics for all allowed interfaces [1 second].");

    loop {
        interval.tick().await;
        // Calling filter_interfaces concurrently.
        let stats = filter_interfaces(handle, &server_config.get_config().interface_filter, server_config).await;

        match stats {
            Ok(stat) => {
                let stats = save_stat(Arc::clone(&last_stats), stat).await;

                if let Some(stat) = stats {
                    add_stat(&client, stat).await.inspect_err(|e| {
                        error!("Failed to save stats: {e}");
                    }).ok();
                }
            },
            Err(e) => error!("Failed to get stats for the filtered interface: {}", e)
        }
    }
}

pub async fn filter_interfaces(handle: &Handle, rules: &Vec<Option<String>>, config: &ServerConfiguration) -> Result<Vec<Stat>, rtnetlink::Error> {
    let compiled_rules: Vec<Option<Regex>> = rules
        .iter()
        .map(|rule_opt| rule_opt.as_ref().and_then(|pattern| Regex::new(pattern).ok()))
        .collect();
    let compiled_rules = Arc::new(compiled_rules);

    // Get all interfaces.
    let interfaces = get_all_interfaces(handle).await?;

    // Process each interface concurrently with a limit of 10.
    let max_concurrent = 10;
    let stats: Vec<Stat> = futures::stream::iter(interfaces)
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
                        get_stats(&handle, &name, &config).await
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
    Ok(stats)
}

pub async fn save_stat(last_stats: Arc<tokio::sync::Mutex<HashMap<String, Option<Stat>>>>, stats: Vec<Stat>) -> Option<Vec<Stat>> {
    let mut final_stats: Vec<Stat> = Vec::new();

    for curr_stat in stats {
        let server_id = curr_stat.server_id.as_str();
        let interface = curr_stat.interface.as_str();

        {
            // Lock the last_stats map to read/update the previous data.
            let mut last_data = last_stats.lock().await;
            if let Some(Some(old_data)) = last_data.get(interface) {
                let old_time = old_data.timestamp;
                let dt = (curr_stat.timestamp - old_time) as u64;

                final_stats.push( Stat {
                    server_id: server_id.into(),
                    interface: interface.into(),
                    timestamp: curr_stat.timestamp,
                    tx_p: (curr_stat.tx_p - old_data.tx_p) / dt,
                    rx_p: (curr_stat.rx_p - old_data.rx_p) / dt,
                    tx: (curr_stat.tx - old_data.tx) / dt,
                    rx: (curr_stat.rx - old_data.rx) / dt,
                    tx_d: (curr_stat.tx_d - old_data.tx_d) / dt,
                    rx_d: (curr_stat.rx_d - old_data.rx_d) / dt,
                });
            }
            // Update the last_stats for this interface.
            last_data.insert(interface.into(), Some(curr_stat));
        }
    }
    Some(final_stats)
}
