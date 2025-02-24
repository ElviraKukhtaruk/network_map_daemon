use klickhouse::{ DateTime, Ipv6 };
use std::net::Ipv6Addr;

#[derive(Debug)]
#[derive(klickhouse::Row)]
pub struct Stat {
    pub server_id: String,
    pub interface: String,
    pub timestamp: DateTime,
    pub rx: u64,
    pub tx: u64,
    pub rx_p: u64,
    pub tx_p: u64,
    pub rx_d: u64,
    pub tx_d: u64
}

#[derive(Debug)]
#[derive(klickhouse::Row)]
pub struct Addr {
    pub server_id: String,
    pub interface: String,
    pub ipv6: Vec<(Option<Ipv6>, Option<u8>)>,
    pub ipv6_peer: Vec<(Option<Ipv6>, Option<u8>)>
}

#[derive(Debug)]
#[derive(klickhouse::Row)]
pub struct Server {
    pub server_id: String,
    pub hostname: String,
    pub icao: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub city: Option<String>,
    pub country: Option<String>
}
