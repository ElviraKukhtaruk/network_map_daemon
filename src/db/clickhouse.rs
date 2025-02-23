use config::{Config, Environment, File};
use klickhouse::*;
use dotenv::dotenv;
use std::{default, env};
use std::path::PathBuf;
use crate::config::pattern;
use clap::Parser;

use std::collections::HashMap;
use serde_json::Value;

pub struct DbConnection {
    client: Client
}

impl DbConnection {

    pub async fn new() -> Self {
        dotenv().ok();

        let cli = pattern::Cli::parse();

        let settings = Config::builder()
            .add_source(File::with_name("Config").required(false))
            .add_source(Environment::with_prefix("CLICKHOUSE").keep_prefix(true).separator("_"))
            .set_default("clickhouse.native_port", 9000.to_string()).expect("Configuration error")
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
            Ok(res) => DbConnection { client: res },
            Err(err) => panic!("Connect error: {:?}", err),
        }

    }

    pub async fn get_client(&self) -> &Client {
        &self.client
    }
}
