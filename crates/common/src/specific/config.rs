use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedConfig {
    pub server: ConfigServer,
    pub auth: AuthConfig,
    pub email: Option<ConfigEmail>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigServer {
    pub name: String,
    pub is_secure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub new_users: bool,
    pub auth_key: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            new_users: false,
            auth_key: Alphanumeric.sample_string(&mut rand::thread_rng(), 48),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigEmail {
    pub display_name: String,
    pub sending_email: String,
    pub contact_email: String,

    pub subject_line: String,

    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_relay: String,
}
