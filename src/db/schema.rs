use klickhouse::{DateTime, Ipv6};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
#[derive(klickhouse::Row)]
pub struct Addr {
    pub server_id: String,
    pub interface: String,
    pub ipv6: Vec<(Option<Ipv6>, Option<u8>)>,
    pub ipv6_peer: Vec<(Option<Ipv6>, Option<u8>)>
}

#[derive(Debug, Clone)]
#[derive(klickhouse::Row)]
pub struct Server {
    pub server_id: String,
    pub hostname: String,
    pub label: String,
    pub lat: f64,
    pub lng: f64,
    pub interface_filter: Vec<Option<String>>,
    pub city: Option<String>,
    pub country: Option<String>
}
