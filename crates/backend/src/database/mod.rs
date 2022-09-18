use crate::{Result, config::Config};
use tokio_postgres::{connect, NoTls, Client};

mod migrations;

pub async fn init(config: &Config) -> Result<Client> {
    let (client, connection) = connect(
        &config.database.url,
        NoTls
    ).await?;

    // Initiate Connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            panic!("Database Connection Error: {}", e);
        }
    });


    migrations::start_initiation(&client).await?;

    Ok(client)
}