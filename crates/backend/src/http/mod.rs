use actix_identity::{CookieIdentityPolicy, IdentityService, Identity};
use actix_web::HttpResponse;
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer, cookie::SameSite};
use common::api::WrappingResponse;

use crate::CliArgs;
use crate::config::get_config;
use crate::database::Database;

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
    ident.forget();

    HttpResponse::TemporaryRedirect()
        .insert_header((header::LOCATION, "/logout"))
        .finish()
}


pub async fn register_http_service(cli_args: &CliArgs, db_data: web::Data<Database>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(get_config().auth.auth_key.as_bytes())
                    .name("librarian-auth")
                    .secure(false)
                    .max_age_secs(60 * 60 * 24 * 365)
                    .same_site(SameSite::Strict)
            ))

            .service(search::public_search)

            // API
            .service(api_route())

            .route(
                "/auth/logout",
                web::get().to(logout)
            )

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

            // Other
            .service(actix_files::Files::new("/", "./app/public").index_file("dist/index.html"))
            .default_service(web::route().to(default_handler))
    })
        .bind(format!("{}:{}", &cli_args.host, cli_args.port))?
        .run()
        .await
}