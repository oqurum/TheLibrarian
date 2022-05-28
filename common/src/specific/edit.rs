use num_enum::{IntoPrimitive, TryFromPrimitive};

#[cfg(feature = "backend")]
use rusqlite::{types::{ValueRef, FromSql, FromSqlResult, ToSqlOutput}, ToSql, Result};




#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum EditOperation {
	Create = 0,
	Delete = 1,
	Modify = 2,
	Merge = 3,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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


#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum EditType {
	Book = 0,
	Person = 1,
	Tag = 2,
	Collection = 3,
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