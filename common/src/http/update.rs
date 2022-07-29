use serde::{Deserialize, Serialize};



#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OptionsUpdate {
    pub server_name: Option<String>,
    pub user_signup: Option<bool>,
}