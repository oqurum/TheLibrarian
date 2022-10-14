use std::error::Error;

use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use tokio_postgres::types::{private::BytesMut, to_sql_checked, FromSql, IsNull, ToSql, Type};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Serialize, Deserialize,
)]
#[repr(u8)]
pub enum EditOperation {
    Create = 0,
    Delete = 1,
    Modify = 2,
    Merge = 3,
}

impl EditOperation {
    pub fn get_name(self) -> &'static str {
        match self {
            Self::Create => "Create",
            Self::Delete => "Delete",
            Self::Modify => "Modify",
            Self::Merge => "Merge",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Serialize, Deserialize,
)]
#[repr(u8)]
pub enum EditStatus {
    Accepted = 0,
    Pending = 1,
    Rejected = 2,
    Failed = 3,
    Cancelled = 4,
    ForceAccepted = 5,
    ForceRejected = 6,
}

impl EditStatus {
    pub fn get_name(self) -> &'static str {
        match self {
            Self::Accepted => "Accepted",
            Self::Pending => "Pending",
            Self::Rejected => "Rejected",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
            Self::ForceAccepted => "Force Accepted",
            Self::ForceRejected => "Force Rejected",
        }
    }

    pub fn is_accepted(self) -> bool {
        matches!(self, Self::Accepted | Self::ForceAccepted)
    }

    pub fn is_rejected(self) -> bool {
        matches!(
            self,
            Self::Rejected | Self::Failed | Self::Cancelled | Self::ForceRejected
        )
    }

    pub fn is_pending(self) -> bool {
        matches!(self, Self::Pending)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Serialize, Deserialize,
)]
#[repr(u8)]
pub enum EditType {
    Book = 0,
    Person = 1,
    Tag = 2,
    Collection = 3,
}

impl EditType {
    pub fn get_name(self) -> &'static str {
        match self {
            Self::Book => "Book",
            Self::Person => "Person",
            Self::Tag => "Tag",
            Self::Collection => "Collection",
        }
    }
}

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, FromPrimitive, IntoPrimitive,
)]
#[repr(u8)]
pub enum ModifyValuesBy {
    #[num_enum(default)]
    Overwrite,
    Append,
    Remove,
}

impl Default for ModifyValuesBy {
    fn default() -> Self {
        Self::Overwrite
    }
}

#[cfg(feature = "backend")]
impl<'a> FromSql<'a> for EditOperation {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Ok(Self::try_from(i16::from_sql(ty, raw)? as u8).unwrap())
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as FromSql>::accepts(ty)
    }
}

#[cfg(feature = "backend")]
impl ToSql for EditOperation {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        (u8::from(*self) as i16).to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

#[cfg(feature = "backend")]
impl<'a> FromSql<'a> for EditStatus {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Ok(Self::try_from(i16::from_sql(ty, raw)? as u8).unwrap())
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as FromSql>::accepts(ty)
    }
}

#[cfg(feature = "backend")]
impl ToSql for EditStatus {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        (u8::from(*self) as i16).to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

#[cfg(feature = "backend")]
impl<'a> FromSql<'a> for EditType {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Ok(Self::try_from(i16::from_sql(ty, raw)? as u8).unwrap())
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as FromSql>::accepts(ty)
    }
}

#[cfg(feature = "backend")]
impl ToSql for EditType {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        (u8::from(*self) as i16).to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}
