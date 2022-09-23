use serde::{Deserialize, Serialize};

use crate::item::member::MemberSettings;



#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OptionsUpdate {
    pub server_name: Option<String>,
    pub user_signup: Option<bool>,

    pub member: Option<MemberSettings>,
}