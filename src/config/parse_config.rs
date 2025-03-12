use std::path::PathBuf;

use serde::{ Deserialize, Serialize };

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerConfig {
    pub clickhouse: Option<Clickhouse>,
    pub server: Option<Server>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Server {
    pub server_id: Option<String>,
    pub hostname: Option<String>,
    pub label: Option<String>,
    pub interface_filter: Vec<Option<String>>,
    pub lat: Option<f32>,
    pub lng: Option<f32>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub priority: Option<u8>,
    pub center: Option<bool>,
    pub logs_path: Option<String>
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Clickhouse {
    hostname: Option<String>,
    user: Option<String>,
    password: Option<String>,
    db: Option<String>,
    port: Option<u32>
}
