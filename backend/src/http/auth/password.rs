// TODO: Better security. Simple Proof of Concept.


use actix_identity::Identity;
use actix_web::web;

use chrono::Utc;
use librarian_common::{api::{ApiErrorResponse, WrappingResponse}, Permissions};
use rand::Rng;
use rand::prelude::ThreadRng;
use serde::{Serialize, Deserialize};

use crate::Error;
use crate::WebResult;
use crate::config::get_config;
use crate::database::Database;
use crate::model::MemberModel;
use crate::model::NewMemberModel;


pub static PASSWORD_PATH: &str = "/auth/password";



#[derive(Serialize, Deserialize)]
pub struct PostPasswordCallback {
	pub email: String,
	pub password: String,
	// TODO: Check for Login vs Signup
}

pub async fn post_password_oauth(
	query: web::Form<PostPasswordCallback>,
	identity: Identity,
	db: web::Data<Database>,
) -> WebResult<web::Json<WrappingResponse<String>>> {
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
			created_at: Utc::now(),
			updated_at: Utc::now(),
		};

		new_member.insert(&db).await?
	};

	super::remember_member_auth(member.id, &identity)?;

	Ok(web::Json(WrappingResponse::new(String::from("success"))))
}

pub fn gen_sample_alphanumeric(amount: usize, rng: &mut ThreadRng) -> String {
	rng.sample_iter(rand::distributions::Alphanumeric)
		.take(amount)
		.map(char::from)
		.collect()
}