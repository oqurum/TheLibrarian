use std::{ops::Deref, fmt::{Display, self}, num::ParseIntError, str::FromStr};

#[cfg(feature = "backend")]
use rusqlite::{Result, types::{FromSql, FromSqlResult, ValueRef, ToSql, ToSqlOutput}};

use serde::{Serialize, Deserialize, Deserializer, Serializer};

use crate::ImageType;


macro_rules! create_single_id {
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

create_single_id!(BookPersonId);
create_single_id!(BookTagId);
create_single_id!(BookId);

create_single_id!(ImageId);

create_single_id!(MemberId);

create_single_id!(PersonId);

create_single_id!(TagId);

create_single_id!(EditId);
create_single_id!(EditCommentId);
create_single_id!(EditVoteId);



// TODO: Macro

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageIdType {
	pub id: usize,
	pub type_of: ImageType, // We don't use the full u8. We use up 4 bits.
}

impl ImageIdType {
	pub fn new_book(value: BookId) -> Self {
		Self {
			id: *value,
			type_of: ImageType::Book,
		}
	}

	pub fn new_person(value: PersonId) -> Self {
		Self {
			id: *value,
			type_of: ImageType::Person,
		}
	}


	fn as_string(&self) -> String {
		format!("{}-{}", self.id, self.type_of.as_num())
	}
}

impl<'de> Deserialize<'de> for ImageIdType {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		Ok(Self::from_str(&String::deserialize(deserializer)?).unwrap())
	}
}

impl Serialize for ImageIdType {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		String::serialize(&self.as_string(), serializer)
	}
}

impl Display for ImageIdType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_string().fmt(f)
	}
}

impl FromStr for ImageIdType {
	type Err = crate::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// TODO: Error handling.
		let split = s.split_once('-')
			.and_then(|(l, r)| Some((l.parse().ok()?, ImageType::from_number(r.parse().ok()?)?)));

		Ok(if let Some((id, type_of)) = split {
			Self { id, type_of }
		} else {
			// TODO: Remove.
			Self { id: 0, type_of: ImageType::Book }
		})
	}
}