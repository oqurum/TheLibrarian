use chrono::{DateTime, Utc};
use tokio_postgres::Client;


use crate::{Result};


pub struct AuthModel {
    pub oauth_token: String,
    pub oauth_token_secret: String,
    pub created_at: DateTime<Utc>,
}

impl AuthModel {
    pub async fn insert(&self, client: &Client) -> Result<()> {
        client.execute(
            "INSERT INTO auths (oauth_token, oauth_token_secret, created_at) VALUES (?1, ?2, ?3)",
            params![
                &self.oauth_token,
                &self.oauth_token_secret,
                self.created_at.timestamp_millis()
            ],
        ).await?;

        Ok(())
    }

    pub async fn remove_by_oauth_token(value: &str, client: &Client) -> Result<bool> {
        Ok(client.execute(
            "DELETE FROM auths WHERE oauth_token = ?1",
            params![ value ],
        ).await? != 0)
    }
}