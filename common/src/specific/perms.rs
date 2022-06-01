use bitflags::bitflags;
use serde::{Serialize, Deserialize};

#[cfg(feature = "backend")]
use rusqlite::{Result, types::{FromSql, FromSqlResult, ValueRef, ToSql, ToSqlOutput}};


bitflags! {
	#[derive(Serialize, Deserialize)]
	pub struct GroupPermissions: u64 {
		const ADMIN 			= 1 << 0;
		const MANAGER 			= 1 << 1;
		const BASIC 			= 1 << 2;
	}

	#[derive(Serialize, Deserialize)]
	pub struct SpecificPermissions: u64 {
		const VIEW 				= 1 << 0;
		const VOTING 			= 1 << 1;
		const COMMENT 			= 1 << 2;
		const EDIT 				= 1 << 3;
		const DELETE 			= 1 << 4;
		const CREATE 			= 1 << 5;
		const FORCE_VOTE 		= 1 << 6;
		const MANAGE_MEMBERS	= 1 << 7;
	}
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Permissions {
	pub group: GroupPermissions,
	// TODO: Determine if Specific perms can be removed to update Group perms
	pub specific: SpecificPermissions,
}

impl Permissions {
	pub fn empty() -> Self {
		Self {
			group: GroupPermissions::empty(),
			specific: SpecificPermissions::empty()
		}
	}

	pub fn basic() -> Self {
		Self {
			group: GroupPermissions::BASIC,
			specific: SpecificPermissions::empty()
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
impl FromSql for Permissions {
	#[inline]
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		let val = String::column_result(value)?;
		let (l, r) = val.split_once('-').unwrap();

		Ok(Self {
			group: GroupPermissions { bits: l.parse().unwrap() },
			specific: SpecificPermissions { bits: r.parse().unwrap() },
		})
	}
}

#[cfg(feature = "backend")]
impl ToSql for Permissions {
	#[inline]
	fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
		Ok(format!("{}-{}", self.group.bits, self.specific.bits).into())
	}
}