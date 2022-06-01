use bitflags::bitflags;
use serde::{Serialize, Deserialize};

#[cfg(feature = "backend")]
use rusqlite::{Result, types::{FromSql, FromSqlResult, ValueRef, ToSql, ToSqlOutput}};


bitflags! {
	#[derive(Serialize, Deserialize)]
	pub struct Permissions: u64 {
		const ADMIN 	= 0b00000001;
		const VIEW 		= 0b00000010;
		const VOTING 	= 0b00000100;
		const COMMENT 	= 0b00001000;
		const EDIT 		= 0b00010000;
		const DELETE 	= 0b00100000;
		const CREATE 	= 0b01000000;
	}
}


impl Permissions {
	pub fn as_basic() -> Self {
		Self::VIEW | Self::VOTING | Self::COMMENT
	}

	pub fn as_editor() -> Self {
		Self::as_basic() | Self::EDIT | Self::DELETE | Self::CREATE
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