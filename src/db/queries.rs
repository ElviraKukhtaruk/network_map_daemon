use log::{info, warn, error};
use klickhouse::*;
use crate::schema::{ Server, Addr, Stat };
use crate::errors::query::QueryErr;

pub async fn add_server(client: &Client, server: Server) -> Result<(), QueryErr> {
    info!("Checking if server with ID '{}' exists in the database...", server.server_id);
    let server_exists: bool;

    let query = format!("
        SELECT * FROM server WHERE server_id = '{}'
        AND hostname = '{}';
    ", server.server_id, server.hostname);
    let servers = client.query_opt::<Server>(query).await;

    match servers {
        Ok(Some(received_server)) => server_exists = received_server.server_id == server.server_id &&
        received_server.hostname == server.hostname,

        Ok(None) => server_exists = false,

        Err(e) => {
            error!("An error occured while checking server '{}' hostname '{}': {}", server.server_id, server.hostname, e);
            server_exists = true;
        }
    }

    if !server_exists {
        let rows = vec![server];
        let server = client.insert_native_block("INSERT INTO server FORMAT native", rows).await;

        match server {
            Err(err) => {
                error!("INSERT INTO server: {:?}", err);
                Err(QueryErr::KlickhouseError)
            },
            _ => {
                info!("Server was added to database!");
                Ok(())
            }
        }
    } else {
        warn!("Server with the server_id {} and hostname {} already exists. Skipping.", server.server_id, server.hostname);
        Ok(())
    }
}

pub async fn add_addr(client: &Client, addrs: Addr) -> Result<(), QueryErr> {
    info!("Checking if the interface '{}' of server {} exists in the database...", addrs.interface, addrs.server_id);
    let int_exists: bool;

    let query = format!("
        SELECT * FROM addr WHERE server_id = '{}' AND
        interface = '{}';
    ", addrs.server_id, addrs.interface);
    let interfaces = client.query_opt::<Addr>(query).await;

    match interfaces {
        Ok(Some(addr)) => int_exists = addr.server_id == addrs.server_id &&
            addr.interface == addrs.interface,

        Ok(None) => int_exists = false,

        Err(e) => {
            error!("An error occured while checking interface '{}': {}", addrs.interface, e);
            int_exists = true;
        }
    }

    if !int_exists {
        let address = Addr {
            server_id: addrs.server_id,
            interface: addrs.interface,
            ipv6: addrs.ipv6,
            ipv6_peer: addrs.ipv6_peer
        };

        let rows = vec![address];
        let address = client.insert_native_block("INSERT INTO addr FORMAT native", rows).await;

        match address {
            Err(err) => {
                error!("INSERT INTO addr: {:?}", err);
                Err(QueryErr::KlickhouseError)
            }
            Ok(_) => Ok(())
        }
    } else {
        warn!("An interface with the same name {} and server ID {} already exists. Skipping.", addrs.interface, addrs.server_id);
        Ok(())
    }
}

pub async fn add_stat(client: &Client, stat: Stat) -> Result<(), QueryErr> {
    let stat = Stat {
        server_id: stat.server_id,
        interface: stat.interface,
        timestamp: stat.timestamp,
        rx: stat.rx,
        tx: stat.tx,
        rx_p: stat.rx_p,
        tx_p: stat.tx_p,
        rx_d: stat.rx_d,
        tx_d: stat.tx_d
    };
    let rows = vec![stat];
    let address = client.insert_native_block("INSERT INTO stat FORMAT native", rows).await;

    match address {
        Err(err) => {
            error!("INSERT INTO stat: {:?}", err);
            Err(QueryErr::KlickhouseError)
        }
        Ok(_) => Ok(())
    }
}
