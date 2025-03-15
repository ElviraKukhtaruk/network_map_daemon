use serde::{Deserialize, Serialize};
use std::net::Ipv6Addr;


#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(clickhouse::Row)]
pub struct Stat {
    pub server_id: String,
    pub interface: String,
    pub timestamp: u32,
    pub rx: u64,
    pub tx: u64,
    pub rx_p: u64,
    pub tx_p: u64,
    pub rx_d: u64,
    pub tx_d: u64,
    pub rx_e: u64,
    pub tx_e: u64
}

#[derive(Hash, Eq, PartialEq)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(clickhouse::Row)]
pub struct Addr {
    pub server_id: String,
    pub interface: String,
    pub ipv6: Vec<(Option<Ipv6Addr>, Option<u8>)>,
    pub ipv6_peer: Vec<(Option<Ipv6Addr>, Option<u8>)>
}

#[derive(PartialEq)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(clickhouse::Row)]
pub struct Server {
    pub server_id: String,
    pub hostname: String,
    pub label: String,
    pub lat: f32,
    pub lng: f32,
    pub interface_filter: Vec<Option<String>>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub priority: Option<u8>,
    pub center: Option<bool>
}
