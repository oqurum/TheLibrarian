#![warn(
	clippy::expect_used,
	// clippy::unwrap_used,
)]

#![allow(clippy::manual_map)]

// TODO: Ping/Pong if currently viewing book. View time. How long been on page. Etc.


use actix_web::web;
use clap::Parser;

pub mod cli;
pub mod database;
pub mod error;
pub mod http;
pub mod metadata;
pub mod model;
mod scheduler;
mod util;

pub use cli::CliArgs;
pub use database::Database;
pub use error::{Result, WebResult, WebError, Error, InternalError};
pub use util::*;

#[actix_web::main]
async fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

	// Initial Register of lazy_static CONFIG.
	config::save_config().await?;

	let db = database::init().await?;
	let db_data = web::Data::new(db);

	scheduler::start(db_data.clone());


	println!("Starting HTTP Server on port {}", cli_args.port);

	http::register_http_service(&cli_args, db_data).await?;

	Ok(())
}