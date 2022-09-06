#![warn(
    clippy::expect_used,
    // clippy::unwrap_used,
)]

#![allow(clippy::manual_map)]


#[macro_export]
macro_rules! params {
    () => {
        &[] as &[&(dyn tokio_postgres::types::ToSql + Sync)]
    };
    ($($param:expr),+ $(,)?) => {
        &[$(&$param as &(dyn tokio_postgres::types::ToSql + Sync)),+] as &[&(dyn tokio_postgres::types::ToSql + Sync)]
    };
}


use actix_web::web;
use clap::Parser;

pub mod cli;
pub mod database;
pub mod error;
pub mod http;
pub mod metadata;
pub mod model;
pub mod storage;
mod scheduler;
mod util;

pub use cli::CliArgs;
pub use error::{Result, WebResult, WebError, Error, InternalError};
pub use util::*;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let cli_args = CliArgs::parse();

    // Initial Register of lazy_static CONFIG. Also updates the config if we changed it.
    config::save_config().await?;

    let config = config::get_config();

    { // Initiate Storage
        *storage::STORE.write().await = storage::Storage::pick_service_from_config(&config.storage).await?;
    }

    let client = database::init(&config).await?;
    let db_data = web::Data::new(client);

    scheduler::start(db_data.clone());


    println!("Starting HTTP Server on port {}", cli_args.port);

    http::register_http_service(&cli_args, db_data).await?;

    Ok(())
}