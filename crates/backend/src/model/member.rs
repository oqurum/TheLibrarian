use lazy_static::lazy_static;
use common_local::{util::serialize_datetime, Permissions};
use chrono::{DateTime, TimeZone, Utc};
use common::MemberId;
use serde::Serialize;
use tokio_postgres::Client;

use crate::Result;

use super::{TableRow, AdvRow, row_to_usize};

lazy_static! {
    pub static ref SYSTEM_MEMBER: MemberModel = MemberModel {
        id: MemberId::from(0),
        name: String::from("System"),
        email: None,
        password: None,
        permissions: Permissions::empty(),
        created_at: Utc.timestamp(0, 0),
        updated_at: Utc.timestamp(0, 0),
    };
}


pub struct NewMemberModel {
    pub name: String,
    pub email: Option<String>,
    pub password: Option<String>,

    pub permissions: Permissions,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberModel {
    pub id: MemberId,

    pub name: String,
    pub email: Option<String>,
    pub password: Option<String>,

    pub permissions: Permissions,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,

    #[serde(serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl TableRow for MemberModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: MemberId::from(row.next::<i64>()? as usize),
            name: row.next()?,
            email: row.next()?,
            password: row.next()?,
            permissions: row.next()?,
            created_at: row.next()?,
            updated_at: row.next()?,
        })
    }
}

impl From<MemberModel> for common_local::Member {
    fn from(value: MemberModel) -> common_local::Member {
        common_local::Member {
            id: value.id,
            name: value.name,
            email: value.email,
            permissions: value.permissions,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}



impl NewMemberModel {
    pub async fn insert(self, db: &Client) -> Result<MemberModel> {
        let row = db.query_one(
            "INSERT INTO member (name, email, password, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
            params![
                &self.name, self.email.as_ref(), self.password.as_ref(), self.permissions,
                self.created_at, self.updated_at
            ]
        ).await?;

        Ok(MemberModel {
            id: MemberId::from(row_to_usize(row)?),
            name: self.name,
            email: self.email,
            password: self.password,
            permissions: self.permissions,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

impl MemberModel {
    pub async fn get_by_email(value: &str, db: &Client) -> Result<Option<Self>> {
        db.query_opt(
            "SELECT * FROM member WHERE email = $1",
            params![ value ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn get_by_id(id: MemberId, db: &Client) -> Result<Option<Self>> {
        if id == 0 {
            Ok(Some(SYSTEM_MEMBER.clone()))
        } else {
            db.query_opt(
                "SELECT * FROM member WHERE id = $1",
                params![ *id as i64 ],
            ).await?.map(Self::from_row).transpose()
        }
    }

    pub async fn get_count(db: &Client) -> Result<usize> {
        row_to_usize(db.query_one("SELECT COUNT(*) FROM member", &[]).await?)
    }

    pub async fn find_all(offset: usize, limit: usize, db: &Client) -> Result<Vec<Self>> {
        let values = db.query(
            "SELECT * FROM member LIMIT $1 OFFSET $2",
            params![ limit as i64, offset as i64 ]
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }
}