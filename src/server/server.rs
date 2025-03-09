use std::process;
use clickhouse::Client;
use log::error;

use crate::db::queries::{self, server_exists, update_server};
use crate::db::schema::Server;

pub async fn add_server_to_database(client: &Client, server: Server) {
    // Check if the server exists
    match server_exists(client, server.clone()).await {
        Ok(exists) => {
            // If it exists, update it
            if exists {
                if let Err(e) = update_server(client, server).await {
                    error!("Failed to update existing server: {e}. Exiting...");
                    process::exit(1);
                }
            } else {
                // Add the server
                if let Err(e) = queries::add_server(client, server).await {
                    error!("Failed to add server to the database: {e}. Exiting...");
                    process::exit(1);
                }
            }
        }
        Err(_) => {
            error!("Failed to check if server exists. Exiting...");
            process::exit(1);
        }
    }
}
