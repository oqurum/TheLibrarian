use common::BookId;
use common_local::{CollectionType, CollectionId, Collection, api::UpdateCollectionModel};
use chrono::{DateTime, Utc};
use tokio_postgres::types::ToSql;

use crate::Result;

use super::{AdvRow, TableRow, BookModel, CollectionItemModel, row_to_usize};

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


impl TableRow for CollectionModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: row.next()?,
            name: row.next()?,
            description: row.next()?,
            type_of: CollectionType::from(row.next::<i16>()? as u8),
            created_at: row.next()?,
            updated_at: row.next()?,
        })
    }
}

impl CollectionModel {
    pub async fn find_by_id(id: CollectionId, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            "SELECT * FROM collection WHERE id = $1",
            params![ id ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn find_books_by_id(id: CollectionId, db: &tokio_postgres::Client) -> Result<Vec<BookModel>> {
        let values = db.query(
            "SELECT * FROM book WHERE id IN (SELECT book_id FROM collection_item WHERE collection_id = $1)",
            params![ id ]
        ).await?;

        values.into_iter().map(BookModel::from_row).collect()
    }

    pub async fn get_all(db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let values = db.query(
            "SELECT * FROM collection",
            &[]
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn update_by_id(collection_id: CollectionId, edit: UpdateCollectionModel, db: &tokio_postgres::Client) -> Result<u64> {
        let mut items = Vec::new();
        // We have to Box because DateTime doesn't return a borrow.
        let mut values = vec![
            &collection_id as &(dyn tokio_postgres::types::ToSql + Sync)
        ];

        if let Some(value) = edit.name.as_ref() {
            items.push("name");
            values.push(value as &(dyn tokio_postgres::types::ToSql + Sync));
        }

        if let Some(value) = edit.description.as_ref() {
            items.push("description");
            values.push(value as &(dyn tokio_postgres::types::ToSql + Sync));
        }

        if let Some(items) = edit.added_books {
            let count = CollectionItemModel::count_by_collection_id(collection_id, db).await?;

            // TODO: Single Query.
            for (idx, book_id) in items.into_iter().enumerate() {
                CollectionItemModel {
                    collection_id,
                    book_id,
                    index: count + idx,
                }.insert(db).await?;
            }
        }

        if items.is_empty() {
            return Ok(0);
        }

        Ok(db.execute(
            &format!(
                "UPDATE collection SET {} WHERE id = $1",
                items.into_iter()
                    .enumerate()
                    .map(|(i, v)| format!("{v} = ?{}", 2 + i))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            &values
        ).await?)
    }


    fn gen_search_query(query: Option<&str>, book_id: Option<BookId>, parameters: &mut Vec<Box<dyn ToSql + Sync>>) -> String {
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
            parameters.push(Box::new(format!("%{}%", &query)) as Box<dyn ToSql + Sync>);
        }


        // Search with specific book

        if let Some(bid) = book_id {
            sql_queries.push("id IN (SELECT collection_id FROM collection_item WHERE book_id = ??)".to_string());
            parameters.push(Box::new(*bid as i64) as Box<dyn ToSql + Sync>);
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
        db: &tokio_postgres::Client
    ) -> Result<Vec<Self>> {
        let mut parameters = vec![
            Box::new(limit as i64) as Box<dyn ToSql + Sync>,
            Box::new(offset as i64) as Box<dyn ToSql + Sync>
        ];

        let mut sql = Self::gen_search_query(query, book_id, &mut parameters);

        sql += " LIMIT $1 OFFSET $2";


        let values = db.query(
            &sql,
            &super::boxed_to_dyn_vec(&parameters)
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn count(
        query: Option<&str>,
        book_id: Option<BookId>,
        db: &tokio_postgres::Client
    ) -> Result<usize> {
        let mut parameters = Vec::new();

        let sql = Self::gen_search_query(query, book_id, &mut parameters).replace("SELECT *", "SELECT COUNT(*)");

        row_to_usize(db.query_one(&sql, &super::boxed_to_dyn_vec(&parameters)).await?)
    }
}


impl NewCollectionModel {
    pub async fn insert(self, db: &tokio_postgres::Client) -> Result<CollectionModel> {
        let now = Utc::now();

        let row = db.query_one(
            "INSERT INTO collection (name, description, type_of, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            params![
                &self.name,
                &self.description,
                u8::from(self.type_of) as i16,
                now,
                now
            ]
        ).await?;

        Ok(CollectionModel {
            id: CollectionId::from(row_to_usize(row)?),

            name: self.name,
            description: self.description,
            type_of: self.type_of,

            created_at: now,
            updated_at: now,
        })
    }
}