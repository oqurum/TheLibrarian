use chrono::{DateTime, NaiveDate, Utc};
use common::{
    api::librarian::{PartialBook, PublicBook},
    get_language_id, get_language_name, BookId, PersonId, ThumbnailStore,
};
use common_local::{
    api::{OrderBy, QueryType},
    util::{serialize_datetime, serialize_datetime_opt, serialize_naivedate_opt},
    DisplayMetaItem, MetadataItemCached,
};
use serde::Serialize;
use std::fmt::Write;
use tokio_postgres::types::ToSql;

use crate::Result;

use super::{row_bigint_to_usize, row_int_to_usize, AdvRow, TableRow, BookIsbnModel};

const FIELDS: &str = "id, title, clean_title, description, rating, thumb_url, cached, is_public, edition_count, available_at, language, created_at, updated_at, deleted_at";

#[derive(Debug, Clone, Serialize)]
pub struct BookModel {
    pub id: BookId,

    pub title: Option<String>,
    pub clean_title: Option<String>,
    pub description: Option<String>,
    pub rating: f64,

    pub thumb_path: ThumbnailStore,

    // TODO: Make table for all tags. Include publisher in it. Remove country.
    pub cached: MetadataItemCached,

    pub is_public: bool,
    pub edition_count: usize,

    #[serde(serialize_with = "serialize_naivedate_opt")]
    pub available_at: Option<NaiveDate>,
    pub language: u16,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_opt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl TableRow for BookModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: BookId::from(row.next::<i32>()? as usize),
            title: row.next()?,
            clean_title: row.next()?,
            description: row.next()?,
            rating: row.next()?,
            thumb_path: ThumbnailStore::from(row.next_opt::<String>()?),
            cached: row
                .next_opt::<String>()?
                .map(MetadataItemCached::from_string)
                .unwrap_or_default(),
            is_public: row.next()?,
            edition_count: row.next::<i64>()? as usize,
            available_at: row.next_opt()?,
            language: get_language_id(row.next()?),
            created_at: row.next()?,
            updated_at: row.next()?,
            deleted_at: row.next_opt()?,
        })
    }
}

// TODO: Consolidate all of these into one or two structs.
impl From<BookModel> for DisplayMetaItem {
    fn from(val: BookModel) -> Self {
        DisplayMetaItem {
            id: val.id,
            title: val.title,
            clean_title: val.clean_title,
            description: val.description,
            rating: val.rating,
            thumb_path: val.thumb_path,
            isbns: None,
            cached: val.cached,
            is_public: val.is_public,
            edition_count: val.edition_count,
            available_at: val.available_at,
            language: val.language,
            created_at: val.created_at,
            updated_at: val.updated_at,
            deleted_at: val.deleted_at,
        }
    }
}

impl From<DisplayMetaItem> for BookModel {
    fn from(val: DisplayMetaItem) -> Self {
        BookModel {
            id: val.id,
            title: val.title,
            clean_title: val.clean_title,
            description: val.description,
            rating: val.rating,
            thumb_path: val.thumb_path,
            cached: val.cached,
            is_public: val.is_public,
            edition_count: val.edition_count,
            available_at: val.available_at,
            language: val.language,
            created_at: val.created_at,
            updated_at: val.updated_at,
            deleted_at: val.deleted_at,
        }
    }
}

impl BookModel {
    pub async fn into_public_book(self, host: &str, author_ids: Vec<usize>, with_isbn: bool, db: &tokio_postgres::Client) -> Result<PublicBook> {
        let isbns = if with_isbn {
            Some(BookIsbnModel::get_all(self.id, db).await?.into_iter().map(|v| v.isbn).collect())
        } else {
            None
        };

        Ok(PublicBook {
            author_ids,
            isbns,

            id: *self.id,
            title: self.title,
            clean_title: self.clean_title,
            description: self.description,
            rating: self.rating,
            thumb_url: self
                .thumb_path
                .as_value()
                .map(|v| format!("{host}/api/v1/image/{v}")),
            display_author_id: self.cached.author_id.map(|v| *v),
            publisher: self.cached.publisher,
            is_public: self.is_public,
            edition_count: self.edition_count,
            available_at: self.available_at,
            language: self.language,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        })
    }

    pub async fn into_partial_book(self, host: &str, with_isbn: bool, db: &tokio_postgres::Client) -> Result<PartialBook> {
        let isbns = if with_isbn {
            Some(BookIsbnModel::get_all(self.id, db).await?.into_iter().map(|v| v.isbn).collect())
        } else {
            None
        };

        Ok(PartialBook {
            isbns,

            id: *self.id,
            title: self.title,
            description: self.description,
            rating: self.rating,
            thumb_url: self
                .thumb_path
                .as_value()
                .map(|v| format!("{host}/api/v1/image/{v}")),
            is_public: self.is_public,
            available_at: self.available_at,
            language: self.language,
        })
    }

    pub async fn get_book_count(db: &tokio_postgres::Client) -> Result<usize> {
        row_bigint_to_usize(db.query_one(r#"SELECT COUNT(*) FROM book"#, &[]).await?)
    }

    pub async fn add_or_update_book(&mut self, db: &tokio_postgres::Client) -> Result<()> {
        let does_book_exist = if self.id != 0 {
            // TODO: Make sure we don't for some use a non-existent id and remove this block.
            Self::get_by_id(self.id, db).await?.is_some()
        } else {
            false
        };

        if does_book_exist {
            self.update_book(db).await?;

            Ok(())
        } else {
            let row = db.query_one(r#"
                INSERT INTO book (
                    title, clean_title, description, rating, thumb_url,
                    cached, is_public, edition_count,
                    available_at, language,
                    created_at, updated_at, deleted_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id"#,
                params![
                    &self.title, &self.clean_title, &self.description, self.rating, self.thumb_path.as_value(),
                    &self.cached.as_string_optional(), self.is_public, self.edition_count as i64,
                    &self.available_at, get_language_name(self.language),
                    self.created_at, self.updated_at, self.deleted_at,
                ]
            ).await?;

            self.id = BookId::from(row_int_to_usize(row)?);

            Ok(())
        }
    }

    pub async fn update_book(&mut self, db: &tokio_postgres::Client) -> Result<()> {
        self.updated_at = Utc::now();

        db.execute(
            r#"
            UPDATE book SET
                title = $2, clean_title = $3, description = $4, rating = $5, thumb_url = $6,
                cached = $7, is_public = $8,
                available_at = $9, language = $10,
                updated_at = $11, deleted_at = $12
            WHERE id = $1"#,
            params![
                *self.id as i32,
                &self.title,
                &self.clean_title,
                &self.description,
                &self.rating,
                self.thumb_path.as_value(),
                &self.cached.as_string_optional(),
                self.is_public,
                &self.available_at,
                get_language_name(self.language),
                &self.updated_at,
                self.deleted_at,
            ],
        )
        .await?;

        Ok(())
    }

    pub async fn get_by_id(id: BookId, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(&format!("SELECT {FIELDS} FROM book WHERE id = $1"), params![*id as i32])
            .await?
            .map(Self::from_row)
            .transpose()
    }

    pub async fn exists_by_isbn(value: &str, db: &tokio_postgres::Client) -> Result<bool> {
        Ok(db
            .query_one(
                "SELECT EXISTS(SELECT id FROM book WHERE isbn_10 = $1 OR isbn_13 = $1)",
                params![value],
            )
            .await?
            .try_get(0)?)
    }

    pub async fn remove_by_id(id: BookId, db: &tokio_postgres::Client) -> Result<u64> {
        Ok(db
            .execute("DELETE FROM book WHERE id = $1", params![*id as i32])
            .await?)
    }

    pub async fn get_book_by(
        offset: usize,
        limit: usize,
        order: OrderBy,
        _only_public: bool,
        person_id: Option<PersonId>,
        db: &tokio_postgres::Client,
    ) -> Result<Vec<Self>> {
        let inner_query = if let Some(pid) = person_id {
            format!("WHERE id IN (SELECT book_id FROM book_person WHERE person_id = {pid})")
        } else {
            String::new()
        };

        let values = db
            .query(
                &format!(
                    "SELECT {FIELDS} FROM book {} ORDER BY id {} LIMIT $1 OFFSET $2",
                    inner_query,
                    order.into_string()
                ),
                params![limit as i64, offset as i64],
            )
            .await?;

        values.into_iter().map(Self::from_row).collect()
    }

    fn gen_search_query(
        qt: &QueryType,
        only_public: bool,
        parameters: &mut Vec<Box<dyn ToSql + Sync>>,
    ) -> String {
        let base_param_len = parameters.len();

        let mut sql = String::from("SELECT * FROM book WHERE ");
        let orig_len = sql.len();

        let mut sql_queries = Vec::new();

        // Only Public

        if only_public {
            sql_queries.push("is_public = ??".to_string());
            parameters.push(Box::new(true) as Box<dyn ToSql + Sync>);
        }

        // Query
        match qt {
            QueryType::Query(orig_query) => {
                // TODO: Separate from here.
                // Check for possible isbn.
                let isbn = if let Some(isbn) =
                    common::parse_book_id(orig_query).into_possible_isbn_value()
                {
                    format!(" OR isbn_10 = '{isbn}' OR isbn_13 = '{isbn}'")
                } else {
                    String::new()
                };

                // TODO: Utilize isbn > title > clean_title > description, and sort
                sql_queries.push(format!(
                    r#"
                    to_tsvector(book.language::regconfig, CONCAT(book.title, ' ', book.cached))
                        @@ websearch_to_tsquery(book.language::regconfig, ??)
                    {isbn}
                "#
                ));
                parameters.push(Box::new(orig_query.to_string()) as Box<dyn ToSql + Sync>);
            }

            // Search with specific person
            &QueryType::Person(pid) => {
                sql_queries.push(
                    "id IN (SELECT book_id FROM book_person WHERE person_id = ??)".to_string(),
                );
                parameters.push(Box::new(*pid as i32) as Box<dyn ToSql + Sync>);
            }

            &QueryType::HasPerson(exists) => {
                if exists {
                    sql_queries.push(String::from("id IN (SELECT book_id FROM book_person)"));
                } else {
                    sql_queries.push(String::from("id NOT IN (SELECT book_id FROM book_person)"));
                }
            }
        }

        let sql_query = sql_queries
            .into_iter()
            .enumerate()
            .map(|(i, v)| v.replace("??", &format!("${}", base_param_len + 1 + i)))
            .collect::<Vec<_>>()
            .join(" AND ");

        sql += &sql_query;

        if sql.len() == orig_len {
            // If sql is still unmodified
            String::from("SELECT * FROM BOOK ")
        } else {
            sql
        }
    }

    pub async fn search_book_list(
        qt: &QueryType,
        offset: usize,
        limit: usize,
        order: OrderBy,
        only_public: bool,
        db: &tokio_postgres::Client,
    ) -> Result<Vec<Self>> {
        let mut parameters = vec![
            Box::new(limit as i64) as Box<dyn ToSql + Sync>,
            Box::new(offset as i64) as Box<dyn ToSql + Sync>,
        ];

        let mut sql = Self::gen_search_query(qt, only_public, &mut parameters)
            .replace("SELECT *", &format!("SELECT {FIELDS}"));

        let _ = write!(
            &mut sql,
            " ORDER BY id {} LIMIT $1 OFFSET $2",
            order.into_string()
        );

        let values = db
            .query(&sql, &super::boxed_to_dyn_vec(&parameters))
            .await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn count_search_book(
        qt: &QueryType,
        only_public: bool,
        db: &tokio_postgres::Client,
    ) -> Result<usize> {
        let mut parameters = Vec::new();

        let sql = Self::gen_search_query(qt, only_public, &mut parameters)
            .replace("SELECT *", "SELECT COUNT(id)");

        row_bigint_to_usize(
            db.query_one(&sql, &super::boxed_to_dyn_vec(&parameters))
                .await?,
        )
    }
}
