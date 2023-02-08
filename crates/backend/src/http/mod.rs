use std::time::Duration;

use actix_identity::{Identity, IdentityMiddleware};
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::HttpResponse;
use actix_web::{cookie::SameSite, web, App, HttpServer};
use common::api::WrappingResponse;

use crate::config::get_config;
use crate::CliArgs;

mod api;
mod auth;
mod search;
pub use self::api::api_route;
pub use self::auth::*;

pub type JsonResponse<V> = web::Json<WrappingResponse<V>>;

// TODO: Convert to async closure (https://github.com/rust-lang/rust/issues/62290)
async fn default_handler() -> impl actix_web::Responder {
    actix_files::NamedFile::open_async("./app/public/dist/index.html").await
}

async fn logout(ident: Identity) -> HttpResponse {
    ident.logout();

    HttpResponse::TemporaryRedirect()
        .insert_header((header::LOCATION, "/logout"))
        .finish()
}

pub async fn register_http_service(
    cli_args: &CliArgs,
    db_data: web::Data<tokio_postgres::Client>,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .wrap(Logger::default())
            .wrap(
                IdentityMiddleware::builder()
                    .login_deadline(Some(Duration::from_secs(60 * 60 * 24 * 365)))
                    .build(),
            )
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(get_config().auth.auth_key.as_bytes()))
                    .cookie_name(String::from("librarian-auth"))
                    .cookie_secure(false)
                    .cookie_same_site(SameSite::Strict)
                    .build(),
            )
            .service(search::public_search_book)
            .service(search::public_search_author)
            // API
            .service(api_route())
            .route("/auth/logout", web::get().to(logout))
            // Password
            .route(
                password::PASSWORD_PATH,
                web::post().to(password::post_password_oauth),
            )
            // Passwordless
            .route(
                passwordless::PASSWORDLESS_PATH,
                web::post().to(passwordless::post_passwordless_oauth),
            )
            .route(
                passwordless::PASSWORDLESS_PATH_CB,
                web::get().to(passwordless::get_passwordless_oauth_callback),
            )
            // Links
            .route(
                external::AUTH_LINK_PATH,
                web::post().to(external::post_oauth_link),
            )
            .route(
                external::AUTH_HANDSHAKE_PATH,
                web::get().to(external::get_oauth_handshake),
            )
            // Other
            .service(actix_files::Files::new("/", "./app/public").index_file("dist/index.html"))
            .default_service(web::route().to(default_handler))
    })
    .bind(format!("{}:{}", &cli_args.host, cli_args.port))?
    .run()
    .await
}
