use bitflags::bitflags;
use serde::{Serialize, Deserialize};

#[cfg(feature = "backend")]
use rusqlite::{Result, types::{FromSql, FromSqlResult, ValueRef, ToSql, ToSqlOutput}};


bitflags! {
	#[derive(Serialize, Deserialize)]
	pub struct Permissions: u64 {
		const ADMIN 			= 1 << 0;

		const VIEW 				= 1 << 1;
		const VOTING 			= 1 << 2;
		const COMMENT 			= 1 << 3;
		const EDIT 				= 1 << 4;
		const DELETE 			= 1 << 5;
		const CREATE 			= 1 << 6;
		const FORCE_VOTE 		= 1 << 7;
		const MANAGE_MEMBERS	= 1 << 8;
	}
}


impl Permissions {
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
		Ok(Self { bits: u64::column_result(value)? })
	}
}

#[cfg(feature = "backend")]
impl ToSql for Permissions {
	#[inline]
	fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
		u64::to_sql(&self.bits)
	}
}