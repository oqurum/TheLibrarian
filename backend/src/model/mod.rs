use std::{ops::Deref, fmt::{Display, self}};

use rusqlite::{Result, types::{FromSql, FromSqlResult, ValueRef, ToSql, ToSqlOutput}};
use serde::{Serialize, Deserialize, Deserializer, Serializer};


mod auth;
mod book;
mod book_person;
mod book_tag;
mod image;
mod member;
mod person;
mod person_alt;
mod tag;

pub use auth::*;
pub use book::*;
pub use book_person::*;
pub use book_tag::*;
pub use self::image::*;
pub use member::*;
pub use person::*;
pub use person_alt::*;
pub use tag::*;


macro_rules! create_id {
	($name:ident) => {
		#[repr(transparent)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
		pub struct $name(usize);

		impl FromSql for $name {
			#[inline]
			fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
				Ok(Self(usize::column_result(value)?))
			}
		}

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

		impl PartialEq<usize> for $name {
			fn eq(&self, other: &usize) -> bool {
				self.0 == *other
			}
		}
	};
}


create_id!(BookPersonId);
create_id!(BookTagId);
create_id!(BookId);
create_id!(ImageId);
create_id!(MemberId);
create_id!(PersonAltId);
create_id!(PersonId);
create_id!(TagId);