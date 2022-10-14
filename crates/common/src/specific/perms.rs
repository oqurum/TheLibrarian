use std::error::Error;

use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "backend")]
use tokio_postgres::types::{private::BytesMut, to_sql_checked, FromSql, IsNull, ToSql, Type};

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct GroupPermissions: u64 {
        const ADMIN             = 1 << 0;
        const MANAGER             = 1 << 1;
        const BASIC             = 1 << 2;
    }

    #[derive(Serialize, Deserialize)]
    pub struct SpecificPermissions: u64 {
        const VIEW                 = 1 << 0;
        const VOTING             = 1 << 1;
        const COMMENT             = 1 << 2;
        const EDIT                 = 1 << 3;
        const DELETE             = 1 << 4;
        const CREATE             = 1 << 5;
        const FORCE_VOTE         = 1 << 6;
        const MANAGE_MEMBERS    = 1 << 7;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Permissions {
    pub group: GroupPermissions,
    // TODO: Determine if Specific perms can be removed to update Group perms
    pub specific: SpecificPermissions,
}

impl Permissions {
    pub fn empty() -> Self {
        Self {
            group: GroupPermissions::empty(),
            specific: SpecificPermissions::empty(),
        }
    }

    pub fn basic() -> Self {
        Self {
            group: GroupPermissions::BASIC,
            specific: SpecificPermissions::as_basic(),
        }
    }

    pub fn editor() -> Self {
        Self {
            group: GroupPermissions::BASIC,
            specific: SpecificPermissions::as_editor(),
        }
    }

    pub fn manager() -> Self {
        Self {
            group: GroupPermissions::MANAGER,
            specific: SpecificPermissions::as_manager(),
        }
    }

    /// Returns true if all of the flags in other are contained within self.
    pub fn contains_group(self, value: GroupPermissions) -> bool {
        self.group.contains(value)
    }

    /// Returns true if all of the flags in other are contained within self.
    pub fn contains_specific(self, value: SpecificPermissions) -> bool {
        self.specific.contains(value)
    }

    /// Returns true if there are flags common to both self and other.
    pub fn intersects_group(self, value: GroupPermissions) -> bool {
        self.group.intersects(value)
    }

    /// Returns true if there are flags common to both self and other.
    pub fn intersects_specific(self, value: SpecificPermissions) -> bool {
        self.specific.intersects(value)
    }

    /// Returns true if all of the flags in other are contained within self.
    pub fn contains_any(self, group: GroupPermissions, specific: SpecificPermissions) -> bool {
        self.group.contains(group) || self.specific.contains(specific)
    }

    /// Returns true if there are flags common to both self and other.
    pub fn intersects_any(self, group: GroupPermissions, specific: SpecificPermissions) -> bool {
        self.group.intersects(group) || self.specific.intersects(specific)
    }

    // Custom

    pub fn is_admin(self) -> bool {
        self.contains_group(GroupPermissions::ADMIN)
    }

    /// Either Group: Admin or Manager
    ///
    /// Specific: Create
    pub fn has_creation_perms(self) -> bool {
        self.intersects_any(
            GroupPermissions::ADMIN | GroupPermissions::MANAGER,
            SpecificPermissions::CREATE,
        )
    }

    /// Either Group: Admin or Manager
    ///
    /// Specific: Delete
    pub fn has_deletion_perms(self) -> bool {
        self.intersects_any(
            GroupPermissions::ADMIN | GroupPermissions::MANAGER,
            SpecificPermissions::DELETE,
        )
    }

    /// Either Group: Admin or Manager
    ///
    /// Specific: Edit
    pub fn has_editing_perms(self) -> bool {
        self.intersects_any(
            GroupPermissions::ADMIN | GroupPermissions::MANAGER,
            SpecificPermissions::EDIT,
        )
    }

    /// Either Group: Admin or Manager
    ///
    /// Specific: Voting
    pub fn has_voting_perms(self) -> bool {
        self.intersects_any(
            GroupPermissions::ADMIN | GroupPermissions::MANAGER,
            SpecificPermissions::VOTING,
        )
    }
}

impl SpecificPermissions {
    pub fn as_basic() -> Self {
        Self::VIEW | Self::VOTING | Self::COMMENT
    }

    pub fn as_editor() -> Self {
        Self::as_basic() | Self::EDIT | Self::DELETE | Self::CREATE
    }

    pub fn as_manager() -> Self {
        Self::as_editor() | Self::FORCE_VOTE | Self::MANAGE_MEMBERS
    }
}

#[cfg(feature = "backend")]
impl<'a> FromSql<'a> for Permissions {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let val = String::from_sql(ty, raw)?;

        let (l, r) = val.split_once('-').unwrap();

        Ok(Self {
            group: GroupPermissions {
                bits: l.parse().unwrap(),
            },
            specific: SpecificPermissions {
                bits: r.parse().unwrap(),
            },
        })
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}

#[cfg(feature = "backend")]
impl ToSql for Permissions {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        format!("{}-{}", self.group.bits, self.specific.bits).to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl<'de> Deserialize<'de> for Permissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = String::deserialize(deserializer)?;

        let (l, r) = val.split_once('-').unwrap();

        Ok(Self {
            group: GroupPermissions {
                bits: l.parse().unwrap(),
            },
            specific: SpecificPermissions {
                bits: r.parse().unwrap(),
            },
        })
    }
}

impl Serialize for Permissions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        format!("{}-{}", self.group.bits, self.specific.bits).serialize(serializer)
    }
}
