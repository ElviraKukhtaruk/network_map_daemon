use std::process;
use clickhouse::Client;
use config::{Config, Environment, File};
use log::{info, error};
use dotenv::dotenv;
use crate::config::parse_cli;
use crate::db::schema::Server;
use clap::Parser;
use crate::config::{ config_file, cli };
use super::get_server_info::get_machine_id;
use crate::config::logs::configure_logs;

pub struct DbConnection {
    client: Client,
    config: Config
}


#[derive(Debug, Clone)]
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
            .set_default("clickhouse.port", "9000").unwrap_or_else(|err| {
                error!("Configuration error: {}", err);
                process::exit(1);
            })
            .set_default("config_path", config_file).unwrap_or_else(|err| {
                error!("Configuration error: {}", err);
                process::exit(1);
            });

        let override_options = [
            ("clickhouse.user", cli.user),
            ("clickhouse.password", cli.password),
            ("clickhouse.db", cli.db),
            ("clickhouse.hostname", cli.servername),
            ("clickhouse.port", cli.port.map(|p| p.to_string())),
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


        let username: String = config.get("clickhouse.user").expect("user key is missing");
        let password: String = config.get("clickhouse.password").expect("password key for clickhouse is missing");
        let default_database: String = config.get("clickhouse.db").expect("db key for clickhouse is missing");


        let host: String = config.get("clickhouse.hostname").expect("hostname key for clickhouse is missing");
        let port: String = config.get("clickhouse.port").expect("port key for clickhouse is missing");

        let socket = format!("http://{host}:{port}/");

        let client = Client::default()
            .with_url(&socket)
            .with_user(username)
            .with_password(password)
            .with_database(default_database);

            DbConnection { client, config }
    }

    pub fn get_client(&self) -> Client {
        self.client.clone()
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }
}


impl ServerConfiguration {

    pub fn new(config: &Config) -> Self {
        let config_file_params = config_file::get_parameters_from_config_file(config);
        let cli_params = cli::get_parameters_from_cli();
        let config_server = config_file_params.and_then(|cfg| cfg.server);

        // Extract all config values at once
        let (config_server_id, config_hostname, config_interface_filter,
            config_label, config_country, config_city, config_lat,
            config_lng, config_priority, config_center) =

        if let Some(s) = config_server {(
            s.server_id, s.hostname, s.interface_filter, s.label,
            s.country, s.city, s.lat, s.lng, s.priority, s.center
        )} else {(
            None, None, Vec::new(), None,
            None, None, None, None, None, None
        )};

        // Construct server fields with CLI taking precedence over config
        let server_id = cli_params.server_id.unwrap_or_else(|| get_machine_id(config_server_id));

        let hostname = cli_params.hostname.or(config_hostname).expect("Missing parameter: hostname");

        let interface_filter = if !cli_params.interface_filter.is_empty() { cli_params.interface_filter }
        else { config_interface_filter };

        let label = cli_params.label.or(config_label).expect("Missing parameter: label");

        let country = cli_params.country.or(config_country);
        let city = cli_params.city.or(config_city);
        let lat = cli_params.lat.or(config_lat).expect("Missing parameter: lat");
        let lng = cli_params.lng.or(config_lng).expect("Missing parameter: lng");
        let priority = cli_params.priority.or(config_priority);
        let center = cli_params.center.or(config_center);

        // Build and return the server configuration
        let server = Server {
            server_id, hostname, label, interface_filter,
            country, city, lat, lng, priority, center
        };

        info!("Server configuration is valid");
        ServerConfiguration { config: server }
    }

    pub fn get_config(&self) -> &Server {
        &self.config
    }
}
