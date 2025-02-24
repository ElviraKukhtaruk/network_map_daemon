use klickhouse::*;
use crate::schema::{ Server, Addr };
use crate::errors::query::QueryErr;
use crate::interface::addr:: { get_addresses };

pub async fn add_server(client: &Client, server: Server) -> Result<(), QueryErr> {

    if server.icao.is_some() || (server.lat.is_some() && server.lng.is_some()) {
        let rows = vec![server];

        let server = client.insert_native_block("INSERT INTO server FORMAT native", rows).await;

        match server {
            Err(err) => {
                eprintln!("INSERT INTO server: {:?}", err);
                Err(QueryErr::KlickhouseError)
            },
            _ => Ok(())
        }
    } else {
        eprintln!("At least the ICAO or lat (latitude) and lng (longitude) parameters are required");
        Err(QueryErr::MissingParametersError)
    }
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
            eprintln!("INSERT INTO addr: {:?}", err);
            Err(QueryErr::KlickhouseError)
        }
        Ok(_) => Ok(())
    }
}
