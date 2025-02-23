use config::{Config, Environment, File};
use klickhouse::*;
use dotenv::dotenv;

pub struct DbConnection {
    client: Client
}

impl DbConnection {

    pub async fn new() -> Self {
        dotenv().ok();

        let settings = Config::builder()
            .add_source(File::with_name("Config").required(false))
            .add_source(Environment::with_prefix("CLICKHOUSE"))
            .build()
            .expect("Configuration error");

        let options = ClientOptions {
            username: settings.get("user").expect("user key is missing"),
            password: settings.get("password").expect("user key for clickhouse is missing"),
            default_database: settings.get("db").expect("user key for clickhouse is missing"),
        };

        let host: String = settings.get("hostname_for_app").expect("hostname_for_app key for clickhouse is missing");
        let port: String = settings.get("native_port").expect("native_port key for clickhouse is missing");

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
