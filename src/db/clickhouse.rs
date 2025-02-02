use config::{Config, File};
use klickhouse::*;

pub async fn connect() -> Client {

    let settings = Config::builder()
        .add_source(File::with_name("config"))
        .build()
        .expect("Config.toml error");

    let options = ClientOptions {
        username: settings.get("clickhouse.user").expect("user key is missing"),
        password: settings.get("clickhouse.password").expect("user key for clickhouse is missing"),
        default_database: settings.get("clickhouse.db").expect("user key for clickhouse is missing"),
    };

    let host: String = settings.get("clickhouse.hostname_for_app").expect("hostname_for_app key for clickhouse is missing");
    let port: String = settings.get("clickhouse.native_port").expect("native_port key for clickhouse is missing");

    let socket = format!("{host}:{port}");

    let client = Client::connect(socket, options).await;

    match client {
        Ok(res) => res,
        Err(err) => panic!("Connect error: {:?}", err),
    }
}
