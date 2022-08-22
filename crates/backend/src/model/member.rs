use lazy_static::lazy_static;
use common_local::{util::serialize_datetime, Permissions};
use chrono::{DateTime, TimeZone, Utc};
use common::MemberId;
use rusqlite::{params, OptionalExtension};
use serde::Serialize;

use crate::{Database, Result};

use super::{TableRow, AdvRow};

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

impl TableRow<'_> for MemberModel {
    fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.next()?,
            name: row.next()?,
            email: row.next()?,
            password: row.next()?,
            permissions: row.next()?,
            created_at: Utc.timestamp_millis(row.next()?),
            updated_at: Utc.timestamp_millis(row.next()?),
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
    pub async fn insert(self, db: &Database) -> Result<MemberModel> {
        let conn = db.write().await;

        conn.execute(r#"
            INSERT INTO members (name, email, password, permissions, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![
            &self.name, self.email.as_ref(), self.password.as_ref(), self.permissions,
            self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
        ])?;

        Ok(MemberModel {
            id: MemberId::from(conn.last_insert_rowid() as usize),
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
    pub async fn get_by_email(value: &str, db: &Database) -> Result<Option<Self>> {
        Ok(db.read().await.query_row(
            r#"SELECT * FROM members WHERE email = ?1"#,
            params![value],
            |v| Self::from_row(v)
        ).optional()?)
    }

    pub async fn get_by_id(id: MemberId, db: &Database) -> Result<Option<Self>> {
        if id == 0 {
            Ok(Some(SYSTEM_MEMBER.clone()))
        } else {
            Ok(db.read().await.query_row(
                r#"SELECT * FROM members WHERE id = ?1"#,
                params![id],
                |v| Self::from_row(v)
            ).optional()?)
        }
    }

    pub async fn get_count(db: &Database) -> Result<usize> {
        Ok(db.read().await.query_row(r#"SELECT COUNT(*) FROM members"#, [], |v| v.get(0))?)
    }

    pub async fn find_all(offset: usize, limit: usize, db: &Database) -> Result<Vec<Self>> {
        let this = db.read().await;

        let mut conn = this.prepare(r#"SELECT * FROM members LIMIT ?1 OFFSET ?2"#)?;

        let map = conn.query_map([limit, offset], |v| Self::from_row(v))?;

        Ok(map.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}