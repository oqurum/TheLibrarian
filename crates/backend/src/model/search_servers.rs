use std::time::Duration;

use chrono::{DateTime, Utc, TimeZone};
use common_local::{ServerLinkId, SearchItemId, util::serialize_datetime};
use serde::Serialize;

use crate::Result;

use super::{TableRow, AdvRow, row_to_usize};


#[derive(Debug)]
pub struct NewSearchItemServerModel {
    pub server_link_id: ServerLinkId,

    pub query: String,
    pub calls: usize,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SearchItemServerModel {
    pub id: SearchItemId,

    pub server_link_id: ServerLinkId,

    pub query: String,
    pub calls: usize,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl TableRow for SearchItemServerModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: row.next()?,

            server_link_id: row.next()?,
            query: row.next()?,
            calls: row.next::<i64>()? as usize,

            created_at: Utc.timestamp_millis(row.next()?),
            updated_at: Utc.timestamp_millis(row.next()?),
        })
    }
}


impl NewSearchItemServerModel {
    pub fn new(server_link_id: ServerLinkId, query: String) -> Self {
        let now = Utc::now();

        Self {
            server_link_id,
            query,
            calls: 1,
            created_at: now,
            updated_at: now,
        }
    }

    pub async fn insert(self, db: &tokio_postgres::Client) -> Result<SearchItemServerModel> {
        let row = db.query_one(
            "INSERT INTO search_item (server_link_id, query, calls, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &self.server_link_id, &self.query, self.calls as i64,
                self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
            ]
        ).await?;

        Ok(SearchItemServerModel {
            id: SearchItemId::from(row_to_usize(row)?),

            server_link_id: self.server_link_id,
            query: self.query,
            calls: self.calls,

            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

    /// Returns a bool to determine if updated
    pub async fn insert_or_inc(self, db: &tokio_postgres::Client) -> Result<bool> {
        if let Some(model) = SearchItemServerModel::find_one_by_server_link_id_and_query(self.server_link_id, &self.query, db).await? {
            // Update if it's been at least 1 hour since last updated.
            if self.updated_at.signed_duration_since(model.updated_at).to_std().unwrap() > Duration::from_secs(60 * 60) {
                SearchItemServerModel::increment_one_by_id(model.id, db).await?;

                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            self.insert(db).await?;
            Ok(true)
        }
    }
}

impl SearchItemServerModel {
    pub async fn find_one_by_server_link_id_and_query(server_link_id: ServerLinkId, query: &str, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT * FROM search_item WHERE server_link_id = ?1 AND query = ?2"#,
            params![ server_link_id, query ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn increment_one_by_id(id: SearchItemId, db: &tokio_postgres::Client) -> Result<u64> {
        Ok(db.execute(
            r#"UPDATE search_item SET calls = calls + 1, updated_at = ?2 WHERE id = ?1"#,
            params![ id, Utc::now().timestamp_millis() ],
        ).await?)
    }

    pub async fn update(&self, db: &tokio_postgres::Client) -> Result<u64> {
        Ok(db.execute(r#"
            UPDATE search_item SET
                server_link_id = ?2,
                query = ?3,
                calls = ?4,
                created_at = ?5,
                updated_at = ?6
            WHERE id = ?1"#,
            params![
                self.id,
                &self.server_link_id, &self.query, self.calls as i64,
                self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
            ]
        ).await?)
    }
}