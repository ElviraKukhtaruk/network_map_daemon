use clickhouse::error::Error;
use futures::future::join_all;
use log::info;
use clickhouse::Client;
use crate::schema::{ Server, Addr, Stat };

pub async fn server_exists(client: &Client, server: Server) -> Result<bool, Error> {
    let servers = client.query("SELECT * FROM server WHERE server_id = ?")
        .bind(server.server_id)
        .fetch_optional::<Server>().await;

    match servers {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(err) => Err(err)
    }
}

pub async fn add_server(client: &Client, server: Server) -> Result<(), Error> {
    let mut insert_server = client.insert("server")?;

    insert_server.write(&server).await?;
    insert_server.end().await?;

    info!("Server was added to database!");
    Ok(())
}

pub async fn update_server(client: &Client, server: Server) -> Result<(), Error> {

    client.query("ALTER TABLE server UPDATE hostname=?, label=?, lat=?,
        lng=?, interface_filter=?, city=?, country=?, priority=?, center=? WHERE server_id=?")
        .bind(&server.hostname)
        .bind(&server.label)
        .bind(&server.lat)
        .bind(&server.lng)
        .bind(&server.interface_filter)
        .bind(&server.city)
        .bind(&server.country)
        .bind(&server.priority)
        .bind(&server.center)
        .bind(&server.server_id)
        .execute().await?;

    info!("Server was updated!");
    Ok(())
}

pub async fn get_addr(client: &Client, server: &Server) -> Result<Vec<Addr>, Error> {

    let addrs = client.query("SELECT * FROM addr WHERE server_id = ?")
        .bind(&server.server_id)
        .fetch_all::<Addr>().await?;

    Ok(addrs)
}

pub async fn add_addr(client: &Client, addrs: Vec<Addr>) -> Result<(), Error> {
    info!("Adding interfaces to the database");

    let mut insert_server = client.insert("addr")?;
    for addr in addrs {
        insert_server.write(&addr).await?;
    }
    insert_server.end().await?;
    Ok(())
}

pub async fn delete_addr(client: &Client, addrs: Vec<Addr>) -> Result<(), Error> {
    info!("Deleting interfaces from the database");

    if addrs.is_empty() {
        return Ok(());
    }

    // Build the WHERE clause dynamically
    let mut query = String::from("DELETE FROM addr WHERE ");
    let conditions: Vec<String> = addrs
        .iter()
        .map(|_| "(server_id = ? AND interface = ?)".to_string())
        .collect();
    query.push_str(&conditions.join(" OR "));

    // Prepare and bind parameters
    let mut prepared_query = client.query(&query);
    for addr in &addrs {
        prepared_query = prepared_query.bind(&addr.server_id).bind(&addr.interface);
    }

    prepared_query.execute().await?;

    Ok(())
}

pub async fn update_addr(client: &Client, addrs: Vec<Addr>) -> Result<(), Error> {
    info!("Updating interfaces");

    if addrs.is_empty() {
        return Ok(());
    }

    // Collect futures for concurrent execution
    let mut update_futures = Vec::new();
    for addr in &addrs {
        let query = "ALTER TABLE addr UPDATE ipv6 = ?, ipv6_peer = ? WHERE server_id = ? AND interface = ?";
        let future = client
            .query(query)
            .bind(&addr.ipv6)
            .bind(&addr.ipv6_peer)
            .bind(&addr.server_id)
            .bind(&addr.interface)
            .execute();
        update_futures.push(future);
    }

    // Execute all updates concurrently and handle results
    let results = join_all(update_futures).await;
    for result in results {
        result?;
    }

    Ok(())
}

pub async fn delete_data_efficiently(client: &Client, server_id: &String) -> Result<(), Error> {
    info!("Deleting data from the addr table");

    client.query("ALTER TABLE addr DROP PARTITION ?")
        .bind(server_id)
        .execute().await?;

    info!("Successfully deleted data for server with ID {server_id} from the addr table");
    Ok(())
}

pub async fn add_stat(client: &Client, stats: Vec<Stat>) -> Result<(), Error> {

    let mut insert_stat = client.insert("stat")?;
    for stat in stats.clone() {
        insert_stat.write(&stat).await?;
    }
    insert_stat.end().await?;
    Ok(())
}
