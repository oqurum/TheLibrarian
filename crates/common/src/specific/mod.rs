pub mod config;
pub mod edit;
mod perms;
mod ids;

use num_enum::{FromPrimitive, IntoPrimitive};
pub use perms::*;
pub use ids::*;
pub use config::*;
use serde::Serialize;





#[derive(Clone, Copy, PartialEq, Eq, Serialize, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MetadataSearchType {
    #[num_enum(default)]
    Book,
    Person,
}

