use log::{error, info};
use clickhouse::Client;
use rtnetlink::{Error, Handle};
use tokio::time::{interval, Duration};
use crate::config::config::ServerConfiguration;
use crate::queries::add_stat;
use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
use crate::db::schema::Stat;
use super::info::{get_filtered_interfaces_names, get_interface_stats};
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
            tx_d: stat.tx_dropped,
            rx_e: stat.rx_error,
            tx_e: stat.tx_error
        });
    }
    None
}

pub async fn save_stats_every_second(handle: &Handle, server_config: &ServerConfiguration, client: &Client) -> Result<(), Error> {
    let stats_interval = Duration::from_secs(1);
    let refresh_interval = Duration::from_secs(60);

    let mut stats_timer = interval(stats_interval);
    let mut refresh_timer = interval(refresh_interval);
    // Cache the interface names initially
    let cached_interface_names = match get_filtered_interfaces_names(handle, &server_config.get_config().interface_filter).await {
        Ok(names) => names,
        Err(e) => {
            error!("Failed to get initial interface names: {e}");
            return Err(e);
        }
    };
    let mut cached_interface_names = cached_interface_names;
    // For concurrent updates
    let last_stats = Arc::new(tokio::sync::Mutex::new(HashMap::<String, Option<Stat>>::new()));

    info!("Collecting and saving statistics every {} second(s).", stats_interval.as_secs());
    info!("Refreshing interface list every {} second(s).", refresh_interval.as_secs());

    loop {
        tokio::select! {
            _ = stats_timer.tick() => {
                // Use the cached interface names for stats collection
                if !cached_interface_names.is_empty() {
                    let stats_result = filter_interfaces(handle, cached_interface_names.clone(), server_config).await;
                    let maybe_stat = save_stat(Arc::clone(&last_stats), stats_result).await;
                    if let Some(stat) = maybe_stat {
                        add_stat(client, stat).await.inspect_err(|e| {
                            error!("Failed to save stats: {e}");
                        }).ok();
                    }
                }
            },
            _ = refresh_timer.tick() => {
                // Refresh the cached interface names periodically
                match get_filtered_interfaces_names(handle, &server_config.get_config().interface_filter).await {
                    Ok(new_interfaces) => {
                        if !new_interfaces.is_empty() {
                            cached_interface_names = new_interfaces;
                        }
                    },
                    Err(e) => {
                        error!("Failed to refresh interface names: {e}, continuing with existing names");
                    }
                }
            }
        }
    }
}

pub async fn filter_interfaces(handle: &Handle, filtered_interface_names: Vec<String>, config: &ServerConfiguration) -> Vec<Stat> {
    let max_concurrent = 10;
    let stats: Vec<Stat> = futures::stream::iter(filtered_interface_names)
        .map(|name| {
            async move {
                get_stats(&handle, &name, &config).await
            }
        })
        .buffer_unordered(max_concurrent)
        .filter_map(|opt| async move { opt })
        .collect()
        .await;

    stats
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

                final_stats.push( Stat {
                    server_id: server_id.into(),
                    interface: interface.into(),
                    timestamp: curr_stat.timestamp,
                    tx_p: (curr_stat.tx_p - old_data.tx_p),
                    rx_p: (curr_stat.rx_p - old_data.rx_p),
                    tx: (curr_stat.tx - old_data.tx),
                    rx: (curr_stat.rx - old_data.rx),
                    tx_d: (curr_stat.tx_d - old_data.tx_d),
                    rx_d: (curr_stat.rx_d - old_data.rx_d),
                    tx_e: (curr_stat.tx_e - old_data.tx_e),
                    rx_e: (curr_stat.rx_e - old_data.rx_e),
                });
            }
            // Update the last_stats for this interface.
            last_data.insert(interface.into(), Some(curr_stat));
        }
    }
    Some(final_stats)
}
