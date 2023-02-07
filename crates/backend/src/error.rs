use common::api::{ApiErrorResponse, WrappingResponse};
use std::fmt::Write;

use std::io::Error as IoError;
use std::time::SystemTimeError;
use std::{num::ParseIntError, sync::PoisonError};

use thiserror::Error as ThisError;

use bcrypt::BcryptError;
use common::error::Error as CommonError;
use common_local::Error as LocalCommonError;
use image::ImageError;
use lettre::address::AddressError;
use lettre::error::Error as LettreError;
use lettre::transport::smtp::Error as SmtpError;
use reqwest::Error as HttpError;
use serde::de::value::Error as SerdeValueError;
use serde_json::Error as JsonError;
use serde_urlencoded::ser::Error as UrlEncodedSerError;
use serde_xml_rs::Error as XmlError;
use tokio_postgres::Error as PostgresError;
use url::ParseError as UrlParseError;

use actix_multipart::MultipartError;
use actix_web::error::PayloadError;
use actix_web::error::UrlencodedError;
use actix_web::Error as ActixError;
use actix_web::ResponseError;

use crate::storage::b2::JsonErrorStruct;

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
    #[error("Urlencoded Error: {0}")]
    Urlencoded(#[from] UrlencodedError),

    #[error(transparent)]
    All(#[from] Error),

    #[error(transparent)]
    LocalCommon(#[from] LocalCommonError),

    #[error(transparent)]
    Common(#[from] CommonError),

    #[error(transparent)]
    ApiResponse(#[from] ApiErrorResponse),
}

impl ResponseError for WebError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let resp_value = match self {
            Self::ApiResponse(r) => WrappingResponse::<()>::Error(r.clone()),

            this => {
                let mut description = String::new();
                let _ = write!(&mut description, "{}", this);
                WrappingResponse::<()>::error(description)
            }
        };

        let mut res = actix_web::HttpResponse::new(self.status_code());

        res.headers_mut().insert(
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );

        res.set_body(actix_web::body::BoxBody::new(
            serde_json::to_string(&resp_value).unwrap(),
        ))
    }
}

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
    #[error("Url Parse Error: {0}")]
    UrlParse(#[from] UrlParseError),

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
    #[error("Postgres Error: {0}")]
    Postgres(#[from] PostgresError),
    #[error("Bcrypt Error: {0}")]
    Bcrypt(#[from] BcryptError),

    #[error("Backblaze Error: {0}")]
    B2(#[from] JsonErrorStruct),

    #[error(transparent)]
    Common(#[from] CommonError),
}

#[derive(Debug, ThisError)]
pub enum InternalError {
    #[error("The user does not exist")]
    UserMissing,

    #[error("The item does not exist")]
    ItemMissing,

    #[error("Unable to convert PathBuf to String")]
    ConvertPathBufToString,

    #[error("Invalid ISBN")]
    InvalidIsbn,

    // Backblaze
    #[error("Backblaze B2 Authorization Error.")]
    B2Authorization,
    #[error("Backblaze B2 Get Upload Url Error.")]
    B2GetUploadUrl,
    #[error("Backblaze B2 Upload File Error.")]
    B2UploadFile,
}

impl<V> From<PoisonError<V>> for Error {
    fn from(_: PoisonError<V>) -> Self {
        Self::Poisoned
    }
}
