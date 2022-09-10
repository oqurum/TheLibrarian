use std::time::Duration;

use chrono::{DateTime, Utc};
use common::Agent;
use common_local::{MetadataSearchId, util::serialize_datetime};
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Serialize, Deserialize};
use tokio_postgres::Client;

use crate::{Result, metadata::{MetadataReturned, SearchItem, AuthorMetadata}};

use super::{TableRow, AdvRow, row_int_to_usize};


pub struct NewMetadataSearchModel {
    pub query: String,
    pub agent: Agent,
    pub type_of: MetadataSearchType,
    pub last_found_amount: usize,
    pub data: DataType,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct MetadataSearchModel {
    pub id: MetadataSearchId,

    pub query: String,
    pub agent: Agent,
    pub type_of: MetadataSearchType,
    pub last_found_amount: usize,
    pub data: String,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl TableRow for MetadataSearchModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: row.next()?,

            query: row.next()?,
            agent: Agent::new_owned(row.next()?),
            type_of: MetadataSearchType::from(row.next::<i16>()? as u8),
            last_found_amount: row.next::<i32>()? as usize,
            data: row.next()?,

            created_at: row.next()?,
            updated_at: row.next()?,
        })
    }
}


impl NewMetadataSearchModel {
    pub fn new(type_of: MetadataSearchType, query: String, agent: Agent, last_found_amount: usize, data: DataType) -> Self {
        let now = Utc::now();

        Self {
            query,
            agent,
            type_of,
            last_found_amount,
            data,
            created_at: now,
            updated_at: now,
        }
    }

    pub async fn insert(self, client: &Client) -> Result<MetadataSearchModel> {
        let data = serde_json::to_string(&self.data)?;

        let row = client.query_one(
            r#"INSERT INTO metadata_search (query, agent, type_of, last_found_amount, data, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"#,
            params![
                &self.query, self.agent.to_string(), u8::from(self.type_of) as i16, self.last_found_amount as i32, &data,
                self.created_at, self.updated_at
            ]
        ).await?;

        Ok(MetadataSearchModel {
            id: MetadataSearchId::from(row_int_to_usize(row)?),

            query: self.query,
            agent: self.agent,
            type_of: self.type_of,
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

    pub async fn find_one_by_query_and_agent(type_of: MetadataSearchType, query: &str, agent: &Agent, client: &Client) -> Result<Option<Self>> {
        client.query_opt(
            "SELECT * FROM metadata_search WHERE type_of = $1 AND query = $2 AND agent = $3",
            params![ u8::from(type_of) as i16, query, agent.to_string() ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn update(&self, client: &Client) -> Result<u64> {
        Ok(client.execute(r#"
            UPDATE metadata_search SET
                query = $2,
                agent = $3,
                type_of = $4
                last_found_amount = $5,
                data = $6,
                created_at = $7,
                updated_at = $8
            WHERE id = $1"#,
            params![
                self.id,
                &self.query, self.agent.to_string(), u8::from(self.type_of) as i16, self.last_found_amount as i32, self.data,
                self.created_at, self.updated_at
            ]
        ).await?)
    }
}



pub struct OptMetadataSearchModel(Option<MetadataSearchModel>);

impl OptMetadataSearchModel {
    pub async fn find_one_by_query_and_agent(type_of: MetadataSearchType, query: &str, agent: &Agent, client: &Client) -> Result<Self> {
        if let Some(model) = MetadataSearchModel::find_one_by_query_and_agent(type_of, query, agent, client).await? {
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

    pub async fn update_or_insert(self, type_of: MetadataSearchType, query: String, agent: Agent, last_found_amount: usize, data: DataType, client: &Client) -> Result<()> {
        if let Some(mut model) = self.0 {
            model.last_found_amount = last_found_amount;
            model.data = serde_json::to_string(&data)?;

            model.update(client).await?;
        } else {
            let model = NewMetadataSearchModel::new(type_of, query, agent, last_found_amount, data);

            model.insert(client).await?;
        }

        Ok(())
    }
}



#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    BookSingle(Option<MetadataReturned>),

    PersonSingle(Option<AuthorMetadata>),

    Search(Vec<SearchItem>),
}

impl DataType {
    pub fn inner_book_single(self) -> Option<MetadataReturned> {
        match self {
            Self::BookSingle(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn inner_person_single(self) -> Option<AuthorMetadata> {
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





#[derive(Clone, Copy, PartialEq, Eq, Serialize, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MetadataSearchType {
    #[num_enum(default)]
    Book,
    Person,
}

