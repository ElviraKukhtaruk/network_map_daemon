use serde::{ Deserialize, Serialize };

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    pub clickhouse: Clickhouse,
    pub server: Server,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Server {
    pub server_id: Option<String>,
    pub hostname: Option<String>,
    pub label: String,
    pub lat: f64,
    pub lng: f64,
    pub city: Option<String>,
    pub country: Option<String>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Clickhouse {
    password: String,
    db: String,
    user: String,
    http_port: u32,
    native_port: u32,
    hostname: String
}
