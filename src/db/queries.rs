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


pub async fn add_addr(client: &Client, addr: Addr) {
   // let prefix_len = &addr.prefix_len;
    let ipv6 = &addr.ipv6;
    let ipv6_peer = &addr.ipv6_peer;

    //if let Some(prefix) = prefix_len {
    //    prefix.is_empty()
    // }
   //



    let rows = vec![addr];

    let address = client.insert_native_block("INSERT INTO addr FORMAT native", rows).await;

    match address {
        Ok(res) => println!("{:?}", res),
        Err(err) => panic!("Select error: {:?}", err),
    }
}
