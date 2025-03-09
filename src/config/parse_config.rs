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
    pub country: Option<String>
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Clickhouse {
    hostname: Option<String>,
    user: Option<String>,
    password: Option<String>,
    db: Option<String>,
    http_port: Option<u32>,
    native_port: Option<u32>
}
