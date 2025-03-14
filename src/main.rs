use interface::get_address::{add_addr_to_database, check_for_interface_updates};
use interface::get_stats::save_stats_every_second;

use server::server::add_server_to_database;
use tokio;
use rtnetlink::{new_connection, Error as rtnetlinkErr, Handle};

mod db;
mod interface;
mod errors;
mod config;
mod server;

use crate::config::config:: { DbConnection, ServerConfiguration };
use crate::db::queries;

use crate::db::schema;


#[tokio::main]
async fn main() -> Result<(), rtnetlinkErr> {
    let con = DbConnection::new().await;
    let server_config = ServerConfiguration::new(con.get_config());
    let get_config = server_config.get_config().clone();

    add_server_to_database(&con.get_client(), get_config).await;

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

   let handle_clone = handle.clone();
   let client_clone = con.get_client().clone();
   let server_conf_clone = server_config.clone();

   add_addr_to_database(&handle, &client_clone, &server_config).await;

   tokio::spawn(async move {
       check_for_interface_updates(&handle_clone, &client_clone, &server_conf_clone).await;
   });

   let stats_task = tokio::spawn(async move {
       save_stats_every_second(&handle, &server_config, &con.get_client()).await;
   });
   stats_task.await.ok();

   Ok(())
}
