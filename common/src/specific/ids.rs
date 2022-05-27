use std::{ops::Deref, fmt::{Display, self}, num::ParseIntError, str::FromStr};

#[cfg(feature = "backend")]
use rusqlite::{Result, types::{FromSql, FromSqlResult, ValueRef, ToSql, ToSqlOutput}};

use serde::{Serialize, Deserialize, Deserializer, Serializer};


macro_rules! create_id {
	($name:ident) => {
		#[repr(transparent)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
		pub struct $name(usize);

		impl $name {
			pub fn none() -> Self {
				Self(0)
			}

			pub fn is_none(self) -> bool {
				self.0 == 0
			}
		}

		#[cfg(feature = "backend")]
		impl FromSql for $name {
			#[inline]
			fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
				Ok(Self(usize::column_result(value)?))
			}
		}

		#[cfg(feature = "backend")]
		impl ToSql for $name {
			#[inline]
			fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
				usize::to_sql(&self.0)
			}
		}

		impl<'de> Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
				Ok(Self(usize::deserialize(deserializer)?))
			}
		}

		impl Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
				usize::serialize(&self.0, serializer)
			}
		}

		impl Deref for $name {
			type Target = usize;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				usize::fmt(&self.0, f)
			}
		}

		impl Default for $name {
			fn default() -> Self {
				Self::none()
			}
		}

		impl PartialEq<usize> for $name {
			fn eq(&self, other: &usize) -> bool {
				self.0 == *other
			}
		}

		impl From<usize> for $name {
			fn from(value: usize) -> Self {
				Self(value)
			}
		}

		impl FromStr for $name {
			type Err = ParseIntError;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				usize::from_str(s).map(Self)
			}
		}
	};
}

create_id!(BookPersonId);
create_id!(BookTagId);
create_id!(BookId);
create_id!(ImageId);
create_id!(MemberId);
create_id!(PersonId);
create_id!(TagId);