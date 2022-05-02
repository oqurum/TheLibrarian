use std::{num::ParseIntError, sync::PoisonError};
use std::io::Error as IoError;
use std::time::SystemTimeError;

use thiserror::Error as ThisError;

use bcrypt::BcryptError;
use rusqlite::Error as RusqliteError;
use lettre::error::Error as LettreError;
use lettre::address::AddressError;
use lettre::transport::smtp::Error as SmtpError;
use image::ImageError;
use reqwest::Error as HttpError;
use serde_urlencoded::ser::Error as UrlEncodedSerError;
use serde_json::Error as JsonError;
use serde_xml_rs::Error as XmlError;
use serde::de::value::Error as SerdeValueError;
use librarian_common::Error as CommonError;

use actix_multipart::MultipartError;
use actix_web::Error as ActixError;
use actix_web::error::PayloadError;
use actix_web::ResponseError;

pub type Result<T> = std::result::Result<T, Error>;
pub type WebResult<T> = std::result::Result<T, WebError>;

// Used specifically for Actix Errors since Actix cannot be used between threads.

#[derive(Debug, ThisError)]
pub enum WebError {
	#[error("ActixWeb Error: {0}")]
	Actix(#[from] ActixError),
	#[error("Multipart Error: {0}")]
	Multipart(#[from] MultipartError),
	#[error("Payload Error: {0}")]
	Payload(#[from] PayloadError),

	#[error(transparent)]
	All(#[from] Error),

	#[error(transparent)]
	Common(#[from] CommonError),
}

impl ResponseError for WebError {}


// Used for all Errors in Application.
#[derive(Debug, ThisError)]
pub enum Error {
	#[error("Internal Error: {0}")]
	Internal(#[from] InternalError),

	#[error("Poison Error")]
	Poisoned,

	#[error("Json Error: {0}")]
	Json(#[from] JsonError),
	#[error("XML Error: {0}")]
	Xml(#[from] XmlError),
	#[error("Serde Value Error: {0}")]
	SerdeValue(#[from] SerdeValueError),
	#[error("Url Encoded Ser Error: {0}")]
	UrlEncodedSer(#[from] UrlEncodedSerError),

	#[error("IO Error: {0}")]
	Io(#[from] IoError),
	#[error("SystemTime Error: {0}")]
	SystemTime(#[from] SystemTimeError),
	#[error("HTTP Error: {0}")]
	Http(#[from] HttpError),
	#[error("Parse Int Error: {0}")]
	ParseInt(#[from] ParseIntError),

	#[error("Image Error: {0}")]
	Image(#[from] ImageError),
	#[error("Lettre Error: {0}")]
	Lettre(#[from] LettreError),
	#[error("SMTP Error: {0}")]
	Smtp(#[from] SmtpError),
	#[error("Address Error: {0}")]
	Address(#[from] AddressError),
	#[error("Rusqlite Error: {0}")]
	Rusqlite(#[from] RusqliteError),
	#[error("Bcrypt Error: {0}")]
	Bcrypt(#[from] BcryptError),

	#[error(transparent)]
	Common(#[from] CommonError),
}

#[derive(Debug, ThisError)]
pub enum InternalError {
	// Actix

	#[error("The user does not exist")]
	UserMissing,
}

impl<V> From<PoisonError<V>> for Error {
	fn from(_: PoisonError<V>) -> Self {
		Self::Poisoned
	}
}