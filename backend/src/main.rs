#![warn(
	clippy::expect_used,
	// clippy::unwrap_used,
)]

#![allow(clippy::manual_map)]

// TODO: Ping/Pong if currently viewing book. View time. How long been on page. Etc.


use actix_web::web;

pub mod config;
pub mod database;
pub mod error;
pub mod http;
pub mod image;
pub mod metadata;

pub use self::image::store_image;
pub use error::{Result, WebResult, WebError, Error, InternalError};


#[actix_web::main]
async fn main() -> Result<()> {
	let db = database::init().await?;

	let db_data = web::Data::new(db);

	println!("Starting HTTP Server on port 8085");

	http::register_http_service(db_data).await?;

	Ok(())
}