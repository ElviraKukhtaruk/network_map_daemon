use std::net::Ipv6Addr;

#[derive(Debug)]
pub struct InterfaceAddr {
    pub address: (Option<Ipv6Addr>, Option<u8>),
    pub local: (Option<Ipv6Addr>, Option<u8>)
}

#[derive(Debug)]
pub struct Stats {
    pub int_name: String,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub tx_dropped: u64,
    pub rx_dropped: u64,
    pub tx_error: u64,
    pub rx_error: u64
}
