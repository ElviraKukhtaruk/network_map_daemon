use std::process;
use interface::get_address::add_interface_addresses;
use interface::get_stats::save_stats_every_second;
use log::error;
use tokio;

use rtnetlink::{new_connection, Error as rtnetlinkErr, Handle};

mod db;
mod interface;
mod errors;
mod config;

use crate::config::config:: { DbConnection, ServerConfiguration };
use crate::db::queries;

use crate::db::schema;

#[tokio::main]
async fn main() -> Result<(), rtnetlinkErr> {
    log4rs::init_file("log4rs.yaml", Default::default()).inspect_err(|err| eprintln!("Logger error: {}", err)).ok();

    let con = DbConnection::new().await;
    let client = con.get_client();
    let server_config = ServerConfiguration::new(con.get_config());
    let get_config = server_config.get_config().clone();

    queries::add_server(client, get_config).await.inspect_err(|_| {
        error!("Failed to add server to the database. Exiting...");
        process::exit(1);
    }).ok();

    // Connection to a Netlink socket
    let connect = new_connection();
    let handle: Handle;

    match connect {
        Ok((connection, get_handle, _)) => {
            handle = get_handle;
            // Running in the background (asynchronously)
            tokio::spawn(connection);
        }
        Err(_) => panic!("RTNetLink Connection failed"),
    }

    add_interface_addresses(&handle, client, &server_config.get_config().interface_filter, &server_config).await
        .inspect_err(|e| error!("An error occurred while getting addresses: {}", e)).ok();

    save_stats_every_second(&handle, &server_config, client).await;
    Ok(())
}
