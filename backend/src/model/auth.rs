use chrono::{DateTime, Utc};


pub struct NewAuthModel {
	pub oauth_token: String,
	pub oauth_token_secret: String,
	pub created_at: DateTime<Utc>,
}