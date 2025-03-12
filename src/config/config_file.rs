use std::{fs, process};
use clap::Parser;
use config::Config;

use super::{parse_cli, parse_config::{ Server, ServerConfig }};
use crate::config::get_server_info::{ get_hostname, get_machine_id };

pub fn read_file(config_path: &str) -> Option<ServerConfig> {
    // Read the existing file
    let content = fs::read_to_string(config_path).inspect_err(|err| {
        eprintln!("An error occured while reading config file: {}", err);
    }).ok()?;
    let config_toml = toml::from_str(&content);

    match config_toml {
        Ok(content) => Some(content),
        Err(err) => {
            eprintln!("An error occured while reading config file: {}", err);
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
                    country: server.country,
                    priority: server.priority,
                    center: server.center,
                    logs_path: server.logs_path
                })
            };

            let updated_toml = toml::to_string_pretty(&final_config).inspect_err(|err| {
                eprintln!("Failed to serialize the given data structure as a TOML string: {}", err);
                process::exit(1);
            });
            if let Ok(toml) = updated_toml {
                fs::write(conf_file, toml).inspect_err(|err| {
                    eprintln!("Failed to write updated TOML string to the configuration file: {}", err);
                    process::exit(1);
                }).ok();
            };
            return Some(final_config);
        } else {
            eprintln!("Configuration file: [server] section is missing.");
        }
    } else {
        eprintln!("Configuration file is missing.");
    }
    None
}
