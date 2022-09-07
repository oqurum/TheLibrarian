use std::{ops::Deref, fmt::{Display, self}, num::ParseIntError, str::FromStr};

use serde::{Serialize, Deserialize, Deserializer, Serializer};

use common::create_single_id;


create_single_id!(EditId);
create_single_id!(EditCommentId);
create_single_id!(EditVoteId);

create_single_id!(ServerLinkId);

create_single_id!(SearchItemId);
create_single_id!(SearchGroupId);

create_single_id!(MetadataSearchId);

create_single_id!(CollectionId);



#[cfg(feature = "backend")]
mod backend {
    use tokio_postgres::types::{FromSql, ToSql, to_sql_checked, Type, private::BytesMut, IsNull};
    use super::*;

    macro_rules! add_sql {
        ($name:ident) => {
            impl<'a> FromSql<'a> for $name {
                fn from_sql(ty: &Type, raw: &'a [u8]) -> std::result::Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                    Ok(Self::from(i32::from_sql(ty, raw)? as usize))
                }

                fn accepts(ty: &Type) -> bool {
                    <i32 as FromSql>::accepts(ty)
                }
            }

            impl ToSql for $name {
                fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> std::result::Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
                    (self.0 as i32).to_sql(ty, out)
                }

                fn accepts(ty: &Type) -> bool {
                    <i32 as ToSql>::accepts(ty)
                }

                to_sql_checked!();
            }
        }
    }

    add_sql!(EditId);
    add_sql!(EditCommentId);
    add_sql!(EditVoteId);
    add_sql!(ServerLinkId);
    add_sql!(SearchItemId);
    add_sql!(SearchGroupId);
    add_sql!(MetadataSearchId);
    add_sql!(CollectionId);
}

#[cfg(feature = "backend")]
pub use backend::*;