use std::str::FromStr;

use bytes::BytesMut;
use chrono::{DateTime, Utc, TimeZone, Datelike};
use common::ImageIdType;
use common_local::{SearchGroupId, util::serialize_datetime, SearchGroup};
use serde::Serialize;
use tokio_postgres::{types::{to_sql_checked, ToSql, FromSql, IsNull, Type}, Client};

use crate::Result;

use super::{TableRow, AdvRow, row_to_usize};


#[derive(Debug)]
pub struct NewSearchGroupModel {
    pub query: String,
    pub calls: usize,
    pub last_found_amount: usize,
    pub timeframe: SearchTimeFrame,
    pub found_id: Option<ImageIdType>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SearchGroupModel {
    pub id: SearchGroupId,

    pub query: String,
    pub calls: usize,
    pub last_found_amount: usize,
    pub timeframe: SearchTimeFrame,
    pub found_id: Option<ImageIdType>,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl TableRow for SearchGroupModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: row.next()?,

            query: row.next()?,
            calls: row.next::<i64>()? as usize,
            last_found_amount: row.next::<i64>()? as usize,
            timeframe: row.next()?,

            found_id: row.next::<Option<String>>()?.map(|v| ImageIdType::from_str(&v)).transpose()?,

            created_at: Utc.timestamp_millis(row.next()?),
            updated_at: Utc.timestamp_millis(row.next()?),
        })
    }
}


impl NewSearchGroupModel {
    pub fn new(query: String, last_found_amount: usize) -> Self {
        let now = Utc::now();

        Self {
            query,
            last_found_amount,
            calls: 1,
            found_id: None,
            timeframe: SearchTimeFrame::now(),
            created_at: now,
            updated_at: now,
        }
    }

    pub async fn insert(self, db: &Client) -> Result<SearchGroupModel> {
        let conn = db;

        let row = conn.query_one(
            "INSERT INTO search_group (query, calls, last_found_amount, timeframe, found_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            params![
                &self.query, self.calls as i64, self.last_found_amount as i64, self.timeframe, self.found_id.as_ref().map(|v| v.to_string()),
                self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
            ]
        ).await?;

        Ok(SearchGroupModel {
            id: SearchGroupId::from(row_to_usize(row)?),

            query: self.query,
            calls: self.calls,
            last_found_amount: self.last_found_amount,
            found_id: self.found_id,

            timeframe: self.timeframe,

            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

    pub async fn insert_or_inc(self, db: &Client) -> Result<SearchGroupModel> {
        if let Some(mut model) = SearchGroupModel::find_one_by_query_and_timeframe(&self.query, self.timeframe, db).await? {
            SearchGroupModel::increment_one_by_id(model.id, self.last_found_amount, db).await?;

            model.calls += 1;
            model.last_found_amount = self.last_found_amount;
            model.updated_at = self.updated_at;

            Ok(model)
        } else {
            self.insert(db).await
        }
    }
}

impl SearchGroupModel {
    pub async fn find_one_by_id(id: SearchGroupId, db: &Client) -> Result<Option<Self>> {
        db.query_opt(
            "SELECT * FROM search_group WHERE id = $1",
            params![ id ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn find_one_by_query_and_timeframe(query: &str, timeframe: SearchTimeFrame, db: &Client) -> Result<Option<Self>> {
        db.query_opt(
            "SELECT * FROM search_group WHERE query = $1 AND timeframe = $2",
            params![ query, timeframe ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn increment_one_by_id(id: SearchGroupId, last_found_amount: usize, db: &Client) -> Result<u64> {
        Ok(db.execute(
            "UPDATE search_group SET calls = calls + 1, updated_at = $2, last_found_amount = $3 WHERE id = $1",
            params![ id, Utc::now().timestamp_millis(), last_found_amount as i64 ],
        ).await?)
    }

    pub async fn update_found_id(id: SearchGroupId, value: Option<ImageIdType>, db: &Client) -> Result<u64> {
        Ok(db.execute(
            "UPDATE search_group SET updated_at = $2, found_id = $3 WHERE id = $1",
            params![ id, Utc::now().timestamp_millis(), value.map(|v| v.to_string()) ],
        ).await?)
    }

    pub async fn get_count(db: &Client) -> Result<usize> {
        row_to_usize(db.query_one("SELECT COUNT(*) FROM search_group", &[]).await?)
    }

    pub async fn find_all(offset: usize, limit: usize, db: &Client) -> Result<Vec<Self>> {
        let values = db.query(
            "SELECT * FROM search_group ORDER BY calls DESC LIMIT $1 OFFSET $2",
            params![ limit as i64, offset as i64 ]
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn update(&self, db: &Client) -> Result<u64> {
        Ok(db.execute(r#"
            UPDATE search_group SET
                query = $2,
                calls = $3,
                last_found_amount = $4,
                timeframe = $5,
                found_id = $6,
                created_at = $7,
                updated_at = $8
            WHERE id = $1"#,
            params![
                self.id,
                &self.query, self.calls as i64, self.last_found_amount as i64, self.timeframe, self.found_id.as_ref().map(|v| v.to_string()),
                self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
            ]
        ).await?)
    }
}


impl From<SearchGroupModel> for SearchGroup {
    fn from(model: SearchGroupModel) -> Self {
        Self {
            id: model.id,
            query: model.query,
            calls: model.calls,
            last_found_amount: model.last_found_amount,
            found_id: model.found_id,
            timeframe: Utc.ymd(model.timeframe.year as i32, model.timeframe.month, 1),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}



#[derive(Debug, Clone, Copy, Serialize)]
pub struct SearchTimeFrame {
    pub year: u32,
    pub month: u32,
}

impl SearchTimeFrame {
    pub fn now() -> Self {
        let now = Utc::now();

        Self {
            year: now.date().year() as u32,
            month: now.date().month() as u32,
        }
    }
}

impl<'a> FromSql<'a> for SearchTimeFrame {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> std::result::Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = i64::from_sql(ty, raw)?;

        Ok(Self {
            year: (value >> 4) as u32,
            month: (value & 0xF) as u32,
        })
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as FromSql>::accepts(ty)
    }
}

impl ToSql for SearchTimeFrame {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> std::result::Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i64::from((self.year << 4) | self.month).to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}