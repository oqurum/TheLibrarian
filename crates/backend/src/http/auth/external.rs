use actix_web::{http::header, web, HttpResponse};
use common::api::{
    librarian::{AuthFormLink, AuthQueryHandshake, Scope},
    reader::VerifyAgentQuery,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    prelude::Distribution,
    thread_rng, Rng,
};
use reqwest::Url;

use crate::{
    model::{NewServerLinkModel, ServerLinkModel},
    WebResult,
};

use super::MemberCookie;

pub static AUTH_LINK_PATH: &str = "/auth/link";
pub static AUTH_HANDSHAKE_PATH: &str = "/auth/handshake";

pub async fn post_oauth_link(
    form: web::Form<AuthFormLink>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<HttpResponse> {
    let member = member.fetch_or_error(&db).await?;

    let mut form = form.into_inner();

    form.server_id = form.server_id.filter(|v| !v.is_empty());

    if form.scope != Scope::ServerRegister && form.server_id.is_none() {
        return Ok(HttpResponse::InternalServerError().body("Missing Server ID"));
    }

    let mut rng = thread_rng();

    let server_id = AlphanumericSpecials.sample_string(&mut rng, 32);
    let public_id = Alphanumeric.sample_string(&mut rng, 40);

    NewServerLinkModel {
        server_owner_name: form.server_owner_name,
        server_name: form.server_name,

        server_id: server_id.clone(),
        public_id: public_id.clone(),

        member_id: member.id,
        verified: false,

        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
    .insert(&db)
    .await?;

    // TODO: Also link account.

    let mut location_uri = Url::parse(&form.redirect_uri).unwrap();
    location_uri.set_query(Some(
        &serde_qs::to_string(&VerifyAgentQuery {
            member_id: *member.id,
            server_id,
            public_id,

            state: form.state,
            scope: form.scope,
        })
        .unwrap(),
    ));

    Ok(HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location_uri.to_string()))
        .body("Redirecting..."))
}

// Uses:
//   - Registering External Servers
//   - Called on server start, periodically to ensure ip routing is correct (ip used for simple connecting through this server)
pub async fn get_oauth_handshake(
    query: web::Query<AuthQueryHandshake>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<HttpResponse> {
    let query = query.into_inner();

    if query.scope == Scope::ServerRegister {
        if let Some(mut link) = ServerLinkModel::get_by_public_id(&query.public_id, &db).await? {
            if link.server_id == query.server_id {
                link.verified = true;

                link.update(&db).await?;

                Ok(HttpResponse::Ok().finish())
            } else {
                Ok(HttpResponse::InternalServerError().body("Not a valid Server id"))
            }
        } else {
            Ok(HttpResponse::InternalServerError().body("Not a valid Public Server ID"))
        }
    } else {
        Ok(HttpResponse::InternalServerError().body("Unknown Scope"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AlphanumericSpecials;

impl Distribution<u8> for AlphanumericSpecials {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
        const RANGE: usize = 26 + 26 + 10 + 8;
        const GEN_ASCII_STR_CHARSET: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%&*~";

        GEN_ASCII_STR_CHARSET[rng.gen_range(0..RANGE)]
    }
}

impl DistString for AlphanumericSpecials {
    fn append_string<R: Rng + ?Sized>(&self, rng: &mut R, string: &mut String, len: usize) {
        unsafe {
            let v = string.as_mut_vec();
            v.extend(self.sample_iter(rng).take(len));
        }
    }
}
