use std::fs;
use clap::Parser;
use config::Config;
use log::{info, error, warn};

use super::{parse_cli, parse_config::{ Server, ServerConfig }};
use crate::config::get_server_info::{ get_hostname, get_machine_id };

pub fn read_file(config_path: &str) -> Option<ServerConfig> {
    // Read the existing file
    let content = fs::read_to_string(config_path).inspect_err(|err| {
        error!("Error occured while reading config file: {}", err);
    }).ok()?;
    let config_toml: Result<ServerConfig, toml::de::Error> = toml::from_str(&content);

    match config_toml {
        Ok(content) => Some(content),
        Err(err) =>{
            error!("Error occured while reading config file: {}", err);
            None
        }
    }
}


pub fn get_parameters_from_config_file(config: &Config) -> Option<ServerConfig> {
    let mut cli = parse_cli::Cli::try_parse();

    let binding = config.get_string("config_path").ok()?;
    let conf_file = binding.as_str();

    let config_exists: bool;

    // Check if the file exists
    config_exists = fs::metadata(conf_file).is_ok();

    if config_exists {
        let config_toml = read_file(conf_file)?;

        let config_machine_id = config_toml.server.as_ref().and_then(|s| s.to_owned().server_id);
        let config_hostname = config_toml.server.as_ref().and_then(|s| s.to_owned().hostname);

        let cli_machine_id = cli.as_mut().ok().and_then(|s| s.server_id.take());
        let cli_hostname = cli.ok().and_then(|s| s.hostname);

        // Override hostname or machine_id with cli parameter
        let hostname = get_hostname(cli_hostname.or_else(|| config_hostname));
        let machine_id = get_machine_id(cli_machine_id.or_else(|| config_machine_id));

        if let Some(server) = config_toml.server {

            let final_config = ServerConfig {
                clickhouse: config_toml.clickhouse,
                server: Some(Server {
                    server_id: Some(machine_id),
                    interface_filter: server.interface_filter,
                    hostname,
                    label: server.label,
                    lat: server.lat,
                    lng: server.lng,
                    city: server.city,
                    country: server.country
                })
            };

            let updated_toml = toml::to_string_pretty(&final_config).inspect_err(|err| {
                error!("Failed to serialize the given data structure as a TOML string: {}", err);
            });

            if let Ok(toml) = updated_toml {
                fs::write(conf_file, toml).inspect_err(|err| {
                    error!("Failed to write updated TOML string to the configuration file: {}", err);
                });
            }
            return Some(final_config);
        } else {
            error!("Configuration file: [server] section is missing.");
        }
    } else {
        error!("Configuration file is missing.");
    }
    None
}
