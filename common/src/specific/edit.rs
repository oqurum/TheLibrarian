use num_enum::{IntoPrimitive, TryFromPrimitive};

#[cfg(feature = "backend")]
use rusqlite::{types::{ValueRef, FromSql, FromSqlResult, ToSqlOutput}, ToSql, Result};
use serde::{Serialize, Deserialize};




#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Serialize, Deserialize)]
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


#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Serialize, Deserialize)]
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
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Serialize, Deserialize)]
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




#[cfg(feature = "backend")]
impl FromSql for EditOperation {
	#[inline]
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		Ok(Self::try_from(u8::column_result(value)?).unwrap())
	}
}

#[cfg(feature = "backend")]
impl ToSql for EditOperation {
	#[inline]
	fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
		Ok(ToSqlOutput::from(u8::from(*self)))
	}
}


#[cfg(feature = "backend")]
impl FromSql for EditStatus {
	#[inline]
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		Ok(Self::try_from(u8::column_result(value)?).unwrap())
	}
}

#[cfg(feature = "backend")]
impl ToSql for EditStatus {
	#[inline]
	fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
		Ok(ToSqlOutput::from(u8::from(*self)))
	}
}


#[cfg(feature = "backend")]
impl FromSql for EditType {
	#[inline]
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		Ok(Self::try_from(u8::column_result(value)?).unwrap())
	}
}

#[cfg(feature = "backend")]
impl ToSql for EditType {
	#[inline]
	fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
		Ok(ToSqlOutput::from(u8::from(*self)))
	}
}