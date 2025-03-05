use futures::StreamExt;
use klickhouse::*;
use regex::Regex;
use rtnetlink::Handle;
use crate::{config::config::ServerConfiguration, db::queries::add_addr};

use super::{addr::get_addresses, info::{get_all_interfaces, get_interface_name_from_attribute, get_loopback_from_header}};


pub async fn add_interface_addresses(
    handle: &Handle,
    client: &Client,
    rules: &Vec<Option<String>>,
    config: &ServerConfiguration) -> Result<(), rtnetlink::Error> {

    // Precompile regexes from rules once (if they exist).
    let compiled_rules: Vec<Option<Regex>> = rules.iter().map(|rule_opt| {
        rule_opt.as_ref().and_then(|pattern| Regex::new(pattern).ok())
    }).collect();

    // Get all interfaces.
    let interfaces = get_all_interfaces(handle).await?;

    // Process interfaces concurrently, limiting concurrency.
    let max_concurrent = 10;
    futures::stream::iter(interfaces)
        .for_each_concurrent(max_concurrent, |interface| {
            let handle = handle;
            let client = client;
            let config = config;
            let compiled_rules = &compiled_rules;
            async move {
                let int_name = get_interface_name_from_attribute(interface.attributes);
                let is_loopback = get_loopback_from_header(interface.header);

                if let Some(name) = int_name.as_ref() {
                    let interface_name_match = compiled_rules.iter().any(|opt_regex| {
                        if let Some(regex) = opt_regex {
                            regex.is_match(name)
                        } else {
                            // Rule is None and interface is not loopback, allow it.
                            !is_loopback
                        }
                    });

                    if interface_name_match && !is_loopback {
                        if let Ok(addr) = get_addresses(&handle, name.into(), &config).await {
                            add_addr(client, addr).await.ok();
                        }
                    }
                }
            }
        }).await;
    Ok(())
}
