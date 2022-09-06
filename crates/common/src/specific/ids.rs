use std::{ops::Deref, fmt::{Display, self}, num::ParseIntError, str::FromStr};

use serde::{Serialize, Deserialize, Deserializer, Serializer};

use common::create_single_id;
use tokio_postgres::types::{FromSql, ToSql, to_sql_checked, Type, private::BytesMut, IsNull};


macro_rules! add_sql {
    ($name:ident) => {
        impl<'a> FromSql<'a> for $name {
            fn from_sql(ty: &Type, raw: &'a [u8]) -> std::result::Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                Ok(Self::from(i64::from_sql(ty, raw)? as usize))
            }

            fn accepts(ty: &Type) -> bool {
                <i64 as FromSql>::accepts(ty)
            }
        }

        impl ToSql for $name {
            fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> std::result::Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
                (self.0 as i64).to_sql(ty, out)
            }

            fn accepts(ty: &Type) -> bool {
                <i64 as ToSql>::accepts(ty)
            }

            to_sql_checked!();
        }
    }
}


create_single_id!(EditId);
create_single_id!(EditCommentId);
create_single_id!(EditVoteId);

create_single_id!(ServerLinkId);

create_single_id!(SearchItemId);
create_single_id!(SearchGroupId);

create_single_id!(MetadataSearchId);

create_single_id!(CollectionId);


add_sql!(EditId);
add_sql!(EditCommentId);
add_sql!(EditVoteId);
add_sql!(ServerLinkId);
add_sql!(SearchItemId);
add_sql!(SearchGroupId);
add_sql!(MetadataSearchId);
add_sql!(CollectionId);