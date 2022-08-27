use chrono::{DateTime, Utc, TimeZone, Datelike};
use common::ImageIdType;
use common_local::{SearchGroupId, util::serialize_datetime, SearchGroup};
use rusqlite::{params, OptionalExtension, types::{FromSql, ToSqlOutput, FromSqlResult, ValueRef, Value}, ToSql};
use serde::Serialize;

use crate::{Database, Result};

use super::{TableRow, AdvRow};


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

impl TableRow<'_> for SearchGroupModel {
    fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.next()?,

            query: row.next()?,
            calls: row.next()?,
            last_found_amount: row.next()?,
            found_id: row.next()?,

            timeframe: row.next()?,

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

    pub async fn insert(self, db: &Database) -> Result<SearchGroupModel> {
        let conn = db.write().await;

        conn.execute(r#"
            INSERT INTO search_group (query, calls, last_found_amount, timeframe, found_id, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            &self.query, self.calls, self.last_found_amount, self.timeframe, &self.found_id,
            self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
        ])?;

        Ok(SearchGroupModel {
            id: SearchGroupId::from(conn.last_insert_rowid() as usize),

            query: self.query,
            calls: self.calls,
            last_found_amount: self.last_found_amount,
            found_id: self.found_id,

            timeframe: self.timeframe,

            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

    pub async fn insert_or_inc(self, db: &Database) -> Result<SearchGroupModel> {
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
    pub async fn find_one_by_id(id: SearchGroupId, db: &Database) -> Result<Option<Self>> {
        Ok(db.read().await.query_row(
            r#"SELECT * FROM search_group WHERE id = ?1"#,
            [id],
            |v| Self::from_row(v)
        ).optional()?)
    }

    pub async fn find_one_by_query_and_timeframe(query: &str, timeframe: SearchTimeFrame, db: &Database) -> Result<Option<Self>> {
        Ok(db.read().await.query_row(
            r#"SELECT * FROM search_group WHERE query = ?1 AND timeframe = ?2"#,
            params![ query, timeframe ],
            |v| Self::from_row(v)
        ).optional()?)
    }

    pub async fn increment_one_by_id(id: SearchGroupId, last_found_amount: usize, db: &Database) -> Result<usize> {
        Ok(db.write().await.execute(
            r#"UPDATE search_group SET calls = calls + 1, updated_at = ?2, last_found_amount = ?3 WHERE id = ?1"#,
            params![ id, Utc::now().timestamp_millis(), last_found_amount ],
        )?)
    }

    pub async fn get_count(db: &Database) -> Result<usize> {
        Ok(db.read().await.query_row(r#"SELECT COUNT(*) FROM search_group"#, [], |v| v.get(0))?)
    }

    pub async fn find_all(offset: usize, limit: usize, db: &Database) -> Result<Vec<Self>> {
        let this = db.read().await;

        let mut conn = this.prepare(r#"SELECT * FROM search_group ORDER BY calls DESC LIMIT ?1 OFFSET ?2"#)?;

        let map = conn.query_map([ limit, offset ], |v| Self::from_row(v))?;

        Ok(map.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub async fn update(&self, db: &Database) -> Result<usize> {
        Ok(db.write().await
        .execute(r#"
            UPDATE search_group SET
                query = ?2,
                calls = ?3,
                last_found_amount = ?4,
                timeframe = ?5,
                found_id = ?6,
                created_at = ?7,
                updated_at = ?8
            WHERE id = ?1"#,
            params![
                self.id,
                &self.query, self.calls, self.last_found_amount, self.timeframe, &self.found_id,
                self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
            ]
        )?)
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

impl FromSql for SearchTimeFrame {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let value = u64::column_result(value)?;

        Ok(Self {
            year: (value >> 4) as u32,
            month: (value & 0xF) as u32,
        })
    }
}

impl ToSql for SearchTimeFrame {
    #[inline]
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(((self.year << 4) | self.month) as i64)))
    }
}