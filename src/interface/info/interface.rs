use klickhouse::Ipv6;

#[derive(Debug)]
pub struct InterfaceAddr {
    pub address: (Option<Ipv6>, Option<u8>),
    pub local: (Option<Ipv6>, Option<u8>)
}

#[derive(Debug)]
pub struct Stats {
    pub int_status: bool,
    pub int_name: String,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub tx_dropped: u64,
    pub rx_dropped: u64
}
