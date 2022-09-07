use chrono::{DateTime, Utc};
use common::MemberId;
use common_local::{ServerLinkId, util::serialize_datetime};
use serde::Serialize;

use crate::Result;

use super::{TableRow, AdvRow, row_int_to_usize};


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

impl TableRow for ServerLinkModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: row.next()?,

            server_owner_name: row.next()?,
            server_name: row.next()?,
            server_id: row.next()?,
            public_id: row.next()?,

            member_id: MemberId::from(row.next::<i32>()? as usize),
            verified: row.next()?,

            created_at: row.next()?,
            updated_at: row.next()?,
        })
    }
}


impl NewServerLinkModel {
    pub async fn insert(self, db: &tokio_postgres::Client) -> Result<ServerLinkModel> {
        let row = db.query_one(r#"
            INSERT INTO server_link (server_owner_name, server_name, server_id, public_id, member_id, verified, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id
        "#,
        params![
            self.server_owner_name.as_ref(), self.server_name.as_ref(), &self.server_id, &self.public_id, *self.member_id as i32, self.verified,
            self.created_at, self.updated_at
        ]).await?;

        Ok(ServerLinkModel {
            id: ServerLinkId::from(row_int_to_usize(row)?),

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
    pub async fn get_by_server_id(value: &str, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT * FROM server_link WHERE server_id = $1"#,
            params![ value ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn does_exist_by_server_id(value: &str, db: &tokio_postgres::Client) -> Result<bool> {
        Ok(db.query_one(
            "SELECT EXISTS(SELECT id FROM server_link WHERE server_id = $1)",
            params![ value ],
        ).await?.try_get(0)?)
    }

    pub async fn get_by_public_id(value: &str, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT * FROM server_link WHERE public_id = $1"#,
            params![ value ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn get_by_id(id: MemberId, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT * FROM server_link WHERE id = $1"#,
            params![ *id as i32 ],
        ).await?.map(Self::from_row).transpose()
    }


    pub async fn update(&self, db: &tokio_postgres::Client) -> Result<u64> {
        Ok(db.execute(r#"
            UPDATE server_link SET
                server_owner_name = $2,
                server_name = $3,
                server_id = $4,
                public_id = $5,
                member_id = $6,
                verified = $7,
                created_at = $8,
                updated_at = $9
            WHERE id = $1"#,
            params![
                self.id,
                self.server_owner_name.as_ref(), self.server_name.as_ref(), &self.server_id, &self.public_id, *self.member_id as i32, self.verified,
                self.created_at, self.updated_at
            ]
        ).await?)
    }
}