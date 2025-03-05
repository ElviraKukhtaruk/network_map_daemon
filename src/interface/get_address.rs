use log::{warn, error};
use klickhouse::*;
use regex::Regex;
use rtnetlink::{Error, Handle};
use crate::{config::config::ServerConfiguration, db::queries::add_addr};
use crate::schema::Addr;

use super::{addr::get_addresses, info::{get_all_interfaces, get_interface_name_from_attribute, get_loopback_from_header}};

pub async fn filter_interfaces(client: &Client, handle: &Handle, rules: &Vec<Option<String>>) -> Result<Option<Addr>, Error> {
    let interfaces = get_all_interfaces(handle).await?;
    let filter_rules = rules.iter().all(|e| e.is_some());

    if !filter_rules {
        warn!("interface_filter not specified, scanning all available interfaces (exclude loopback).");
    }


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
                let addrs = get_addresses(handle, name.into()).await.ok();
                return Ok(addrs);
            }
        }
    }
    Ok(None)
}

pub async fn save_addresses(handle: &Handle, server_config: &ServerConfiguration, client: &Client) {

    let addresses = filter_interfaces(&client, &handle, &server_config.get_config().interface_filter).await
        .inspect_err(|err| error!("Failed to get interfaces stats: {}", err));


}
