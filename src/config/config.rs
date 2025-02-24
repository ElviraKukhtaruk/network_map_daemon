use config::{Config, Environment, File};
use klickhouse::*;
use dotenv::dotenv;
use crate::config::pattern;
use clap::Parser;
use std::{fs, io::Read};
use rand::{self, RngCore};
use hex;
use toml;
use crate::config::parse::{ ServerConfig, Server };

pub struct DbConnection {
    client: Client,
    config: Config
}


#[derive(Debug)]
pub struct ServerConfiguration {
    config: ServerConfig
}

fn get_hostname(hostname: Option<String>) -> String {
    match hostname {
        Some(hostname) => hostname,
        None => {
            let hostname = fs::read_to_string("/etc/hostname")
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|err| {
                    panic!("Can't get hostname from: /etc/hostname: {}", err)
            });
            hostname
        }
    }
}

fn get_machine_id(machine_id: Option<String>) -> String {

    let machine_id = machine_id.or_else(|| {
        fs::read_to_string("/etc/machine-id")
        .map(|s| s.trim().to_string())
        .inspect_err(|err| eprintln!("Can't get machine-id from /etc/machine-id: {}. Generating new one.", err))
        .ok()
    }).unwrap_or_else(|| {
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    });
    machine_id
}


impl DbConnection {

    pub async fn new() -> Self {
        dotenv().ok();

        let cli = pattern::Cli::parse();

        let mut config_file = String::from("Config.toml");

        // Set custom path to config file
        if let Some(config) = cli.config.as_deref() {
            config_file = config.display().to_string();
        }

        let settings = Config::builder()
            .add_source(File::with_name(config_file.as_str()).required(false))
            .add_source(Environment::with_prefix("CLICKHOUSE").keep_prefix(true).separator("_"))
            .set_default("clickhouse.native_port", 9000.to_string()).expect("Configuration error")
            .set_default("config_path", config_file).expect("Configuration error")
            .set_override_option("clickhouse.user", cli.user.as_deref().map(|p| p.to_string_lossy().into_owned())).expect("Configuration error")
            .set_override_option("clickhouse.password", cli.password.as_deref()).expect("Configuration error")
            .set_override_option("clickhouse.db", cli.db.as_deref()).expect("Configuration error")
            .set_override_option("clickhouse.hostname", cli.servername.as_deref()).expect("Configuration error")
            .set_override_option("clickhouse.native_port", cli.native_port);


        let config = if let Ok(builder) = settings {
            builder.build().expect("Configuration error")
        } else {
            panic!("Configuration error");
        };

        let options = ClientOptions {
            username: config.get("clickhouse.user").expect("user key is missing"),
            password: config.get("clickhouse.password").expect("user key for clickhouse is missing"),
            default_database: config.get("clickhouse.db").expect("user key for clickhouse is missing"),
        };

        let host: String = config.get("clickhouse.hostname").expect("hostname_for_app key for clickhouse is missing");
        let port: String = config.get("clickhouse.native_port").expect("native_port key for clickhouse is missing");

        let socket = format!("{host}:{port}");

        let client = Client::connect(socket, options).await;

        match client {
            Ok(res) => DbConnection { client: res, config },
            Err(err) => panic!("Connect error: {:?}", err),
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

        let conf_file = config.get_string("config_path").expect("Config path not specified!");

        // Check if the file exists
        let mut config_string = String::new();
        let config_exists = fs::metadata(conf_file.as_str()).is_ok();

        if config_exists {
            // Read the existing file
            let mut file = fs::OpenOptions::new().read(true).open(conf_file.as_str()).unwrap_or_else(|err| {
                panic!("Error occured while opening config file: {}", err);
            });
            file.read_to_string(&mut config_string).unwrap_or_else(|err| {
                panic!("Error while reading config file: {}", err);
            });

            // Parse existing TOML
            let config_toml: Result<ServerConfig, toml::de::Error> = toml::from_str(&config_string.as_str());

            match config_toml {
                Ok(res) => {

                    let hostname = get_hostname(res.server.hostname);

                    let machine_id = get_machine_id(res.server.server_id);

                   let final_config = ServerConfig {
                        clickhouse: res.clickhouse,
                        server: Server {
                            server_id: Some(machine_id),
                            hostname: Some(hostname),
                            label: res.server.label,
                            lat: res.server.lat,
                            lng: res.server.lng,
                            city: res.server.city,
                            country: res.server.country
                        }
                    };
                    let updated_toml = toml::to_string_pretty(&final_config).unwrap_or_else(|err| {
                        panic!("Failed to generate server_id and hostname for configuration file: {}", err);
                    });

                    fs::write(conf_file, updated_toml).unwrap_or_else(|err| {
                        panic!("Failed to generate server_id and hostname for configuration file: {}", err);
                    });

                    ServerConfiguration { config: final_config }
                },
                Err(e) => panic!("Configuration error: {:?}", e.message())
            }
        } else {
            panic!("Configuration file is missing: {}", conf_file);
        }
    }

    pub fn get_config(&self) -> &ServerConfig {
        &self.config
    }
}
