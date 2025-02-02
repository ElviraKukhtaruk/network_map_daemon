use std::net::IpAddr;

#[derive(Debug)]
pub struct IntterfaceAddr {
    pub prefix_len: u8,
    pub address: IpAddr,
    pub local: IpAddr
}

#[derive(Debug)]
pub struct Stats {
    pub int_status: bool,
    pub int_name: String,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64
}