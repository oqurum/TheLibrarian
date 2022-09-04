use common::BookId;
use common_local::{CollectionType, CollectionId, Collection, api::UpdateCollectionModel};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, OptionalExtension, ToSql};

use crate::{Database, Result};

use super::{AdvRow, TableRow};

pub struct NewCollectionModel {
    pub name: String,
    pub description: Option<String>,
    pub type_of: CollectionType,
}


pub struct CollectionModel {
    pub id: CollectionId,

    pub name: String,
    pub description: Option<String>,
    pub type_of: CollectionType,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


impl From<CollectionModel> for Collection {
    fn from(val: CollectionModel) -> Self {
        Collection {
            id: val.id,
            name: val.name,
            description: val.description,
            type_of: val.type_of,
            created_at: val.created_at,
            updated_at: val.updated_at
        }
    }
}


impl TableRow<'_> for CollectionModel {
    fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.next()?,
            name: row.next()?,
            description: row.next()?,
            type_of: CollectionType::from(row.next::<u8>()?),
            created_at: Utc.timestamp_millis(row.next()?),
            updated_at: Utc.timestamp_millis(row.next()?),
        })
    }
}

impl CollectionModel {
    pub async fn find_by_id(id: CollectionId, db: &Database) -> Result<Option<Self>> {
        Ok(db.read().await.query_row(
            r#"SELECT * FROM collection WHERE id = ?1"#,
            params![id],
            |v| Self::from_row(v)
        ).optional()?)
    }

    pub async fn get_all(db: &Database) -> Result<Vec<Self>> {
        let this = db.read().await;

        let mut conn = this.prepare("SELECT * FROM collection")?;

        let map = conn.query_map([], |v| Self::from_row(v))?;

        Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub async fn update_by_id(id: CollectionId, edit: UpdateCollectionModel, db: &Database) -> Result<usize> {
        let mut items = Vec::new();
        // We have to Box because DateTime doesn't return a borrow.
        let mut values = vec![
            &id as &dyn rusqlite::ToSql
        ];

        if let Some(value) = edit.name.as_ref() {
            items.push("name");
            values.push(value as &dyn rusqlite::ToSql);
        }

        if let Some(value) = edit.description.as_ref() {
            items.push("description");
            values.push(value as &dyn rusqlite::ToSql);
        }

        // if let Some(value) = edit.items {}

        if items.is_empty() {
            return Ok(0);
        }

        Ok(db.write().await
        .execute(
            &format!(
                "UPDATE collection SET {} WHERE id = ?1",
                items.into_iter()
                    .enumerate()
                    .map(|(i, v)| format!("{v} = ?{}", 2 + i))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            rusqlite::params_from_iter(values.iter())
        )?)
    }


    fn gen_search_query(query: Option<&str>, book_id: Option<BookId>, parameters: &mut Vec<Box<dyn ToSql>>) -> String {
        let base_param_len = parameters.len();

        let mut sql = String::from("SELECT * FROM collection WHERE ");
        let orig_len = sql.len();

        let mut sql_queries = Vec::new();

        // Query

        if let Some(orig_query) = query.as_ref() {
            // Used to escape percentages
            let mut escape_char = '\\';
            // Change our escape character if it's in the query.
            if orig_query.contains(escape_char) {
                for car in [ '!', '@', '#', '$', '^', '&', '*', '-', '=', '+', '|', '~', '`', '/', '?', '>', '<', ',' ] {
                    if !orig_query.contains(car) {
                        escape_char = car;
                        break;
                    }
                }
            }

            let query = orig_query.replace('%', &format!("{}%", escape_char))
                .replace('_', &format!("{}_", escape_char));

            // TODO: Utilize title > description and sort
            sql_queries.push(format!("name LIKE ?? ESCAPE '{escape_char}'"));
            parameters.push(Box::new(format!("%{}%", &query)) as Box<dyn ToSql>);
        }


        // Search with specific book

        if let Some(bid) = book_id {
            sql_queries.push("id IN (SELECT collection_id FROM collection_item WHERE book_id = ??)".to_string());
            parameters.push(Box::new(bid) as Box<dyn ToSql>);
        }

        let sql_query = sql_queries.into_iter()
            .enumerate()
            .map(|(i, v)| v.replace("??", &format!("?{}", base_param_len + 1 + i)))
            .collect::<Vec<_>>()
            .join(" AND ");

        sql += &sql_query;

        // If sql is still unmodified
        if sql.len() == orig_len {
            String::from("SELECT * FROM collection")
        } else {
            sql
        }
    }

    pub async fn search(
        query: Option<&str>,
        offset: usize,
        limit: usize,
        book_id: Option<BookId>,
        db: &Database
    ) -> Result<Vec<Self>> {
        let mut parameters = vec![
            Box::new(limit) as Box<dyn ToSql>,
            Box::new(offset) as Box<dyn ToSql>
        ];

        let mut sql = Self::gen_search_query(query, book_id, &mut parameters);

        sql += " LIMIT ?1 OFFSET ?2";

        let this = db.read().await;

        let mut conn = this.prepare(&sql)?;

        let map = conn.query_map(rusqlite::params_from_iter(parameters), |v| Self::from_row(v))?;

        Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub async fn count(
        query: Option<&str>,
        book_id: Option<BookId>,
        db: &Database
    ) -> Result<usize> {
        let mut parameters = Vec::new();

        let sql = Self::gen_search_query(query, book_id, &mut parameters).replace("SELECT *", "SELECT COUNT(*)");

        Ok(db.read().await.query_row(&sql, rusqlite::params_from_iter(parameters), |v| v.get(0))?)
    }
}


impl NewCollectionModel {
    pub async fn insert(self, db: &Database) -> Result<CollectionModel> {
        let conn = db.write().await;

        let now = Utc::now();

        conn.execute(r#"
            INSERT INTO collection (name, description, type_of, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![
            &self.name,
            &self.description,
            u8::from(self.type_of),
            now.timestamp_millis(),
            now.timestamp_millis()
        ])?;

        Ok(CollectionModel {
            id: CollectionId::from(conn.last_insert_rowid() as usize),

            name: self.name,
            description: self.description,
            type_of: self.type_of,

            created_at: now,
            updated_at: now,
        })
    }
}