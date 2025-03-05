use std::process;

use config::{Config, Environment, File};
use log::{info, error, warn};
use klickhouse::*;
use dotenv::dotenv;
use crate::config::parse_cli;
use crate::db::schema::Server;
use clap::Parser;
use crate::config::{ config_file, cli };
use super::get_server_info::{get_hostname, get_machine_id};

pub struct DbConnection {
    client: Client,
    config: Config
}


#[derive(Debug)]
pub struct ServerConfiguration {
    config: Server
}


impl DbConnection {

    pub async fn new() -> Self {
        dotenv().ok();

        let cli = parse_cli::Cli::parse();

        let mut config_file = String::from("Config.toml");

        // Set custom path to config file
        if let Some(config) = cli.config.as_deref() {
            config_file = config.display().to_string();
        }

        let mut settings = Config::builder()
            .add_source(File::with_name(config_file.as_str()).required(false))
            .add_source(Environment::with_prefix("CLICKHOUSE").keep_prefix(true).separator("_"))
            .set_default("clickhouse.native_port", "9000").unwrap_or_else(|err| {
                error!("Configuration error: {}", err);
                process::exit(1);
            })
            .set_default("config_path", config_file).unwrap_or_else(|err| {
                error!("Configuration error: {}", err);
                process::exit(1);
            });

        let override_options = [
            ("clickhouse.user", cli.user.as_deref().map(|p| p.to_string_lossy().into())),
            ("clickhouse.password", cli.password),
            ("clickhouse.db", cli.db),
            ("clickhouse.hostname", cli.servername),
            ("clickhouse.native_port", cli.native_port.map(|p| p.to_string())),
        ];

        for (key, value) in override_options {
            settings = settings.set_override_option(key, value).unwrap_or_else(|err| {
                error!("Configuration error: {}", err);
                process::exit(1);
            });
        }
        let config = settings.build().unwrap_or_else(|err| {
            error!("Configuration error: {}", err);
            process::exit(1);
        });

        let options = ClientOptions {
            username: config.get("clickhouse.user").expect("user key is missing"),
            password: config.get("clickhouse.password").expect("user key for clickhouse is missing"),
            default_database: config.get("clickhouse.db").expect("user key for clickhouse is missing"),
        };

        let host: String = config.get("clickhouse.hostname").expect("hostname_for_app key for clickhouse is missing");
        let port: String = config.get("clickhouse.native_port").expect("native_port key for clickhouse is missing");

        let socket = format!("{host}:{port}");

        let client = Client::connect(&socket, options).await;

        match client {
            Ok(res) => {
                info!("Connected to clickhouse: {}", &socket);
                DbConnection { client: res, config }
            },
            Err(err) => {
                error!("Connect error: {:?}", err);
                process::exit(1);
            }
        }

    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }
}


impl ServerConfiguration {

    pub fn new(config: &Config) -> Self {
        let config_file_params = config_file::get_parameters_from_config_file(config);
        let cli_params = cli::get_parameters_from_cli();
        let mut config_server = config_file_params.and_then(|cfg| cfg.server);

        let machine_id = config_server.as_mut().and_then(|s| s.server_id.take());
        let hostname = config_server.as_mut().and_then(|s| s.hostname.take());

        let interface_filter = if !cli_params.interface_filter.is_empty() {
            cli_params.interface_filter
        } else {
            config_server
                .as_mut()
                .map(|s| std::mem::take(&mut s.interface_filter))
                .unwrap_or_else(Vec::new)
        };

        let server = Server {
            server_id: cli_params.server_id.unwrap_or_else(|| get_machine_id(machine_id)),
            hostname: cli_params.hostname.or_else(|| get_hostname(hostname)).expect("Missing parameter: hostname"),
            label: cli_params.label.or_else(|| config_server.as_mut().and_then(|s| s.label.take())).expect("Missing parameter: label"),
            interface_filter,
            country: cli_params.country.or_else(|| config_server.as_mut().and_then(|s| s.country.take())),
            city: cli_params.city.or_else(|| config_server.as_mut().and_then(|s| s.city.take())),
            lat: cli_params.lat.or_else(|| config_server.as_mut().and_then(|s| s.lat.take())).expect("Missing parameter: lat"),
            lng: cli_params.lng.or_else(|| config_server.as_mut().and_then(|s| s.lng.take())).expect("Missing parameter: lng"),
        };
        info!("Server configuration is valid");
        ServerConfiguration { config: server }
    }

    pub fn get_config(&self) -> &Server {
        &self.config
    }
}
