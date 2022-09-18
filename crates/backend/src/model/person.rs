use common_local::{Person, util::{serialize_datetime, serialize_naivedate_opt}};
use chrono::{DateTime, Utc, NaiveDate};
use common::{BookId, PersonId, Source, ThumbnailStore, api::librarian::PublicAuthor};
use serde::Serialize;

use crate::Result;

use super::{PersonAltModel, TableRow, AdvRow, row_int_to_usize, row_bigint_to_usize};



#[derive(Debug)]
pub struct NewPersonModel {
    pub source: Source,

    pub name: String,
    pub description: Option<String>,
    pub birth_date: Option<NaiveDate>,

    pub thumb_url: ThumbnailStore,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PersonModel {
    pub id: PersonId,

    pub source: Source,

    pub name: String,
    pub description: Option<String>,
    #[serde(serialize_with = "serialize_naivedate_opt")]
    pub birth_date: Option<NaiveDate>,

    pub thumb_url: ThumbnailStore,

    #[serde(serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
}

impl TableRow for PersonModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: PersonId::from(row.next::<i32>()? as usize),

            source: Source::try_from(row.next::<String>()?).unwrap(),

            name: row.next()?,
            description: row.next()?,
            birth_date: row.next_opt()?,

            thumb_url: ThumbnailStore::from(row.next_opt::<String>()?),

            created_at: row.next()?,
            updated_at: row.next()?,
        })
    }
}

impl From<PersonModel> for Person {
    fn from(val: PersonModel) -> Self {
        Person {
            id: val.id,
            source: val.source,
            name: val.name,
            description: val.description,
            birth_date: val.birth_date,
            thumb_url: val.thumb_url,
            updated_at: val.updated_at,
            created_at: val.created_at,
        }
    }
}


impl NewPersonModel {
    pub async fn insert(self, db: &tokio_postgres::Client) -> Result<PersonModel> {
        let row = db.query_one(
            "INSERT INTO person (source, name, description, birth_date, thumb_url, updated_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            params![
                self.source.to_string(), &self.name, &self.description, &self.birth_date, self.thumb_url.as_value(),
                self.updated_at, self.created_at
            ]
        ).await?;

        Ok(PersonModel {
            id: PersonId::from(row_int_to_usize(row)?),
            source: self.source,
            name: self.name,
            description: self.description,
            birth_date: self.birth_date,
            thumb_url: self.thumb_url,
            updated_at: self.updated_at,
            created_at: self.created_at,
        })
    }
}


impl PersonModel {
    pub fn into_public_author(self, host: &str) -> PublicAuthor {
        PublicAuthor {
            id: *self.id,
            name: self.name,
            description: self.description,
            birth_date: self.birth_date,
            thumb_url: format!("{}/api/v1/image/{}", host, self.thumb_url.as_value().unwrap()),
            updated_at: self.updated_at,
            created_at: self.created_at,
        }
    }

    pub async fn get_all(offset: usize, limit: usize, db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let values = db.query(
            "SELECT * FROM person ORDER BY name ASC LIMIT $1 OFFSET $2",
            params![ limit as i64, offset as i64 ]
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn get_all_by_book_id(book_id: BookId, db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let values = db.query(
            r#"
                SELECT person.* FROM book_person
                LEFT JOIN
                    person ON person.id = book_person.person_id
                WHERE book_id = $1
            "#,
            params![ *book_id as i32 ]
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn search_count(query: &str, db: &tokio_postgres::Client) -> Result<usize> {
        let mut escape_char = '\\';

        // Change our escape character if it's in the query.
        if query.contains(escape_char) {
            for car in [ '!', '@', '#', '$', '^', '&', '*', '-', '=', '+', '|', '~', '`', '/', '?', '>', '<', ',' ] {
                if !query.contains(car) {
                    escape_char = car;
                    break;
                }
            }
        }

        let sql = format!(
            r#"SELECT COUNT(*) FROM person WHERE name ILIKE '%{}%' ESCAPE '{}' ORDER BY name ASC LIMIT $1 OFFSET $2"#,
            query.replace('%', &format!("{}%", escape_char)).replace('_', &format!("{}_", escape_char)),
            escape_char
        );

        row_bigint_to_usize(db.query_one(
            &sql,
            &[]
        ).await?)
    }

    // TODO: Improve
    pub async fn search(query: &str, offset: usize, limit: usize, db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let mut escape_char = '\\';

        // Change our escape character if it's in the query.
        if query.contains(escape_char) {
            for car in [ '!', '@', '#', '$', '^', '&', '*', '-', '=', '+', '|', '~', '`', '/', '?', '>', '<', ',' ] {
                if !query.contains(car) {
                    escape_char = car;
                    break;
                }
            }
        }

        let sql = format!(
            r#"SELECT * FROM person WHERE name ILIKE '%{}%' ESCAPE '{}' ORDER BY name ASC LIMIT $1 OFFSET $2"#,
            query.replace('%', &format!("{}%", escape_char)).replace('_', &format!("{}_", escape_char)),
            escape_char
        );


        let values = db.query(
            &sql,
            params![ limit as i64, offset as i64 ]
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }

    // TODO: Change result to Vec since multiple people can have the same name.
    pub async fn find_one_by_name(value: &str, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        let person = db.query_opt(
            "SELECT * FROM person WHERE name = $1 LIMIT 1",
            params![ value ],
        ).await?;

        if let Some(person) = person {
            Ok(Some(Self::from_row(person)?))
        } else if let Some(alt) = PersonAltModel::get_by_name(value, db).await? {
            Self::get_by_id(alt.person_id, db).await
        } else {
            Ok(None)
        }
    }

    pub async fn get_by_id(id: PersonId, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT * FROM person WHERE id = $1"#,
            params![ *id as i32 ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn get_by_source(value: &str, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            "SELECT * FROM person WHERE source = $1",
            params![ value ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn get_count(db: &tokio_postgres::Client) -> Result<usize> {
        row_bigint_to_usize(db.query_one("SELECT COUNT(*) FROM person", &[]).await?)
    }

    pub async fn update(&self, db: &tokio_postgres::Client) -> Result<()> {
        db.execute(r#"
            UPDATE person SET
                source = $2,
                name = $3,
                description = $4,
                birth_date = $5,
                thumb_url = $6,
                updated_at = $7,
                created_at = $8
            WHERE id = $1"#,
            params![
                *self.id as i32,
                self.source.to_string(), &self.name, &self.description, &self.birth_date, self.thumb_url.as_value(),
                self.updated_at, self.created_at
            ]
        ).await?;

        Ok(())
    }

    pub async fn remove_by_id(id: PersonId, db: &tokio_postgres::Client) -> Result<u64> {
        Ok(db.execute(
            "DELETE FROM person WHERE id = $1",
            params![ *id as i32 ]
        ).await?)
    }
}