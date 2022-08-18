use chrono::{DateTime, Utc, TimeZone};
use common::MemberId;
use common_local::{ServerLinkId, util::serialize_datetime};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;

use crate::{Database, Result};

use super::{TableRow, AdvRow};


#[derive(Debug)]
pub struct NewServerLinkModel {
    pub server_owner_name: Option<String>,
    pub server_name: Option<String>,
    pub server_id: String,
    pub public_id: String,

    pub member_id: MemberId,
    pub verified: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ServerLinkModel {
    pub id: ServerLinkId,

    pub server_owner_name: Option<String>,
    pub server_name: Option<String>,
    pub server_id: String,
    pub public_id: String,

    pub member_id: MemberId,
    pub verified: bool,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl TableRow<'_> for ServerLinkModel {
    fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.next()?,

            server_owner_name: row.next()?,
            server_name: row.next()?,
            server_id: row.next()?,
            public_id: row.next()?,

            member_id: row.next()?,
            verified: row.next()?,

            created_at: Utc.timestamp_millis(row.next()?),
            updated_at: Utc.timestamp_millis(row.next()?),
        })
    }
}


impl NewServerLinkModel {
    pub async fn insert(self, db: &Database) -> Result<ServerLinkModel> {
        let conn = db.write().await;

        conn.execute(r#"
            INSERT INTO server_link (server_owner_name, server_name, server_id, public_id, member_id, verified, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            self.server_owner_name.as_ref(), self.server_name.as_ref(), &self.server_id, &self.public_id, self.member_id, self.verified,
            self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
        ])?;

        Ok(ServerLinkModel {
            id: ServerLinkId::from(conn.last_insert_rowid() as usize),

            server_owner_name: self.server_owner_name,
            server_name: self.server_name,
            server_id: self.server_id,
            public_id: self.public_id,

            member_id: self.member_id,
            verified: self.verified,

            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

impl ServerLinkModel {
    pub async fn get_by_server_id(value: &str, db: &Database) -> Result<Option<Self>> {
        Ok(db.read().await.query_row(
            r#"SELECT * FROM server_link WHERE server_id = ?1"#,
            [value],
            |v| Self::from_row(v)
        ).optional()?)
    }

    pub async fn get_by_public_id(value: &str, db: &Database) -> Result<Option<Self>> {
        Ok(db.read().await.query_row(
            r#"SELECT * FROM server_link WHERE public_id = ?1"#,
            [value],
            |v| Self::from_row(v)
        ).optional()?)
    }

    pub async fn get_by_id(id: MemberId, db: &Database) -> Result<Option<Self>> {
        Ok(db.read().await.query_row(
            r#"SELECT * FROM server_link WHERE id = ?1"#,
            [id],
            |v| Self::from_row(v)
        ).optional()?)
    }


    pub async fn update(&self, db: &Database) -> Result<usize> {
        Ok(db.write().await
        .execute(r#"
            UPDATE server_link SET
                server_owner_name = ?2,
                server_name = ?3,
                server_id = ?4,
                public_id = ?5,
                member_id = ?6,
                verified = ?7,
                created_at = ?8,
                updated_at = ?9
            WHERE id = ?1"#,
            params![
                self.id,
                self.server_owner_name.as_ref(), self.server_name.as_ref(), &self.server_id, &self.public_id, self.member_id, self.verified,
                self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
            ]
        )?)
    }
}