use std::fs;
use rand::{self, RngCore};
use log::error;

pub fn get_hostname(hostname: Option<String>) -> Option<String> {

    // Find hostname in /etc/hostname (if not set), if not found, return None
    if let None = hostname {
        return fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .inspect_err(|err| {
                error!("Can't get hostname from: /etc/hostname: {}", err)
            }).ok();
    }
    hostname
}

pub fn get_machine_id(machine_id: Option<String>) -> String {
    // Find machine-id in /etc/machine-id (if not set), if not found, generate random id.
    let machine_id = machine_id.or_else(|| {
        fs::read_to_string("/etc/machine-id")
        .map(|s| s.trim().to_string())
        .inspect_err(|err| error!("Can't get machine-id from /etc/machine-id: {}. Generating new one.", err))
        .ok()
    }).unwrap_or_else(|| {
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    });
    machine_id
}
