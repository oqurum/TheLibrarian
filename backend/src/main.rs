#![warn(
	clippy::expect_used,
	// clippy::unwrap_used,
)]

#![allow(clippy::manual_map)]

// TODO: Ping/Pong if currently viewing book. View time. How long been on page. Etc.


use actix_web::web;

pub mod database;
pub mod error;
pub mod http;
pub mod metadata;
pub mod model;
mod util;

pub use database::Database;
pub use error::{Result, WebResult, WebError, Error, InternalError};
pub use util::*;

#[actix_web::main]
async fn main() -> Result<()> {
	// Initial Register of lazy_static CONFIG.
	config::save_config().await?;

	let db = database::init().await?;

	let db_data = web::Data::new(db);

	println!("Starting HTTP Server on port 8085");

	http::register_http_service(db_data).await?;

	Ok(())
}