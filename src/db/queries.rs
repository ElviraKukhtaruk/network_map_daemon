use futures::StreamExt;
use log::{info, warn, error};
use klickhouse::*;
use crate::schema::{ Server, Addr, Stat };
use crate::errors::query::QueryErr;

pub async fn add_server(client: &Client, server: Server) -> Result<(), QueryErr> {

    info!("Checking if server with ID '{}' exists in the database...", server.server_id);
    let mut server_exists = false;
    let servers = client.query::<Server>("SELECT * FROM server;").await;

    if let Ok(mut server) = servers {
        while let Some(row) = server.next().await {
            row.inspect_err(|e| error!("SELECT server_id FROM server: {}", e)).ok();
            server_exists = true;
            warn!("Server with the same ID already exists, skipping.");
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

    } else { Ok(()) }
}

pub async fn add_addr(client: &Client, addrs: Addr) -> Result<(), QueryErr> {
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
