// TODO: Better security. Simple Proof of Concept.

use actix_identity::Identity;
use actix_web::web;

use chrono::Utc;
use common::api::ApiErrorResponse;
use common::api::WrappingResponse;
use common_local::Permissions;
use rand::prelude::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::get_config;
use crate::http::JsonResponse;
use crate::model::MemberModel;
use crate::model::NewMemberModel;
use crate::Error;
use crate::WebResult;

pub static PASSWORD_PATH: &str = "/auth/password";

#[derive(Serialize, Deserialize)]
pub struct PostPasswordCallback {
    pub email: String,
    pub password: String,
    // TODO: Check for Login vs Signup
}

pub async fn post_password_oauth(
    query: web::Json<PostPasswordCallback>,
    identity: Identity,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<String>> {
    if identity.identity().is_some() {
        return Err(ApiErrorResponse::new("Already logged in").into());
    }

    let PostPasswordCallback { email, password } = query.into_inner();

    // Create or Update User.
    let member = if let Some(value) = MemberModel::get_by_email(&email, &db).await? {
        if bcrypt::verify(&password, value.password.as_ref().unwrap()).map_err(Error::from)? {
            value
        } else {
            return Err(ApiErrorResponse::new("Unable to very password hash for account").into());
        }
    } else if !get_config().auth.new_users {
        return Err(ApiErrorResponse::new("New user creation is disabled").into());
    } else {
        let hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err(Error::from)?;

        let new_member = NewMemberModel {
            // TODO: Strip email
            name: email.clone(),
            email: Some(email),
            password: Some(hash),
            permissions: Permissions::basic(),
            localsettings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        new_member.insert(&db).await?
    };

    super::remember_member_auth(member.id, &identity)?;

    Ok(web::Json(WrappingResponse::okay(String::from("success"))))
}

pub fn gen_sample_alphanumeric(amount: usize, rng: &mut ThreadRng) -> String {
    rng.sample_iter(rand::distributions::Alphanumeric)
        .take(amount)
        .map(char::from)
        .collect()
}
