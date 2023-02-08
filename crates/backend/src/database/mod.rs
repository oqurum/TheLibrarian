use crate::{config::Config, Result};
use tokio_postgres::{connect, Client, NoTls};

mod migrations;

pub async fn init(config: &Config) -> Result<Client> {
    let (mut client, connection) = connect(&config.database.url, NoTls).await?;

    // Initiate Connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("Database Connection Error: {}", e);

            std::process::exit(1);
        }
    });

    migrations::start_initiation(&mut client).await?;

    Ok(client)
}
