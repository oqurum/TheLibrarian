use std::time::Duration;

use chrono::{DateTime, Utc, TimeZone};
use common_local::{MetadataSearchId, util::serialize_datetime};
use rusqlite::{params, OptionalExtension};
use serde::{Serialize, Deserialize};

use crate::{Database, Result, metadata::{MetadataReturned, SearchItem, AuthorInfo}};

use super::{TableRow, AdvRow};


#[derive(Debug)]
pub struct NewMetadataSearchModel {
    pub query: String,
    pub agent: String,
    pub last_found_amount: usize,
    pub data: DataType,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MetadataSearchModel {
    pub id: MetadataSearchId,

    pub query: String,
    pub agent: String,
    pub last_found_amount: usize,
    pub data: String,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl TableRow<'_> for MetadataSearchModel {
    fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.next()?,

            query: row.next()?,
            agent: row.next()?,
            last_found_amount: row.next()?,
            data: row.next()?,

            created_at: Utc.timestamp_millis(row.next()?),
            updated_at: Utc.timestamp_millis(row.next()?),
        })
    }
}


impl NewMetadataSearchModel {
    pub fn new(query: String, agent: String, last_found_amount: usize, data: DataType) -> Self {
        let now = Utc::now();

        Self {
            query,
            last_found_amount,
            agent,
            data,
            created_at: now,
            updated_at: now,
        }
    }

    pub async fn insert(self, db: &Database) -> Result<MetadataSearchModel> {
        let conn = db.write().await;

        let data = serde_json::to_string(&self.data)?;

        conn.execute(r#"
            INSERT INTO metadata_search (query, agent, last_found_amount, data, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![
            &self.query, self.agent, self.last_found_amount, &data,
            self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
        ])?;

        Ok(MetadataSearchModel {
            id: MetadataSearchId::from(conn.last_insert_rowid() as usize),

            query: self.query,
            agent: self.agent,
            last_found_amount: self.last_found_amount,
            data,

            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

impl MetadataSearchModel {
    /// Can be updated every 7 days
    pub fn can_be_updated(&self) -> bool {
        if let Ok(dur) = Utc::now().signed_duration_since(self.updated_at).to_std() {
            dur > Duration::from_secs(60 * 60 * 24 * 7)
        } else {
            false
        }
    }

    pub fn parse_data(&self) -> Result<DataType> {
        Ok(serde_json::from_str(&self.data)?)
    }

    pub async fn find_one_by_query_and_agent(query: &str, agent: &str, db: &Database) -> Result<Option<Self>> {
        Ok(db.read().await.query_row(
            r#"SELECT * FROM metadata_search WHERE query = ?1 AND agent = ?2"#,
            [ query, agent ],
            |v| Self::from_row(v)
        ).optional()?)
    }

    pub async fn update(&self, db: &Database) -> Result<usize> {
        Ok(db.write().await
        .execute(r#"
            UPDATE metadata_search SET
                query = ?2,
                agent = ?3,
                last_found_amount = ?4,
                data = ?5,
                created_at = ?6,
                updated_at = ?7
            WHERE id = ?1"#,
            params![
                self.id,
                &self.query, self.agent, self.last_found_amount, self.data,
                self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
            ]
        )?)
    }
}



pub struct OptMetadataSearchModel(Option<MetadataSearchModel>);

impl OptMetadataSearchModel {
    pub async fn find_one_by_query_and_agent(query: &str, agent: &str, db: &Database) -> Result<Self> {
        if let Some(model) = MetadataSearchModel::find_one_by_query_and_agent(query, agent, db).await? {
            Ok(Self(Some(model)))
        } else {
            Ok(Self(None))
        }
    }

    /// If we have an existing model and we cannot update it return the cached version.
    pub fn should_use_cached(&self) -> Result<Option<DataType>> {
        self.0.as_ref()
            .filter(|v| !v.can_be_updated())
            .map(|v| v.parse_data())
            .transpose()
    }

    pub async fn update_or_insert(self, query: String, agent: String, last_found_amount: usize, data: DataType, db: &Database) -> Result<()> {
        if let Some(mut model) = self.0 {
            model.last_found_amount = last_found_amount;
            model.data = serde_json::to_string(&data)?;

            model.update(db).await?;
        } else {
            let model = NewMetadataSearchModel::new(query, agent, last_found_amount, data);

            model.insert(db).await?;
        }

        Ok(())
    }
}



#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    BookSingle(Option<MetadataReturned>),

    PersonSingle(Option<AuthorInfo>),

    Search(Vec<SearchItem>),
}

impl DataType {
    pub fn inner_book_single(self) -> Option<MetadataReturned> {
        match self {
            Self::BookSingle(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn inner_person_single(self) -> Option<AuthorInfo> {
        match self {
            Self::PersonSingle(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn inner_search(self) -> Vec<SearchItem> {
        match self {
            Self::Search(v) => v,
            _ => unreachable!(),
        }
    }
}
