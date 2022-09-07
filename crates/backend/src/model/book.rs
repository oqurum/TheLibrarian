use common_local::{MetadataItemCached, DisplayMetaItem, util::{serialize_datetime, serialize_datetime_opt, serialize_naivedate_opt}, search::PublicBook};
use chrono::{DateTime, Utc, NaiveDate};
use common::{ThumbnailStore, BookId, PersonId};
use serde::Serialize;
use tokio_postgres::types::ToSql;

use crate::Result;

use super::{TableRow, AdvRow, row_int_to_usize, row_bigint_to_usize};


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

    pub isbn_10: Option<String>,
    pub isbn_13: Option<String>,

    pub is_public: bool,
    pub edition_count: usize,

    #[serde(serialize_with = "serialize_naivedate_opt")]
    pub available_at: Option<NaiveDate>,
    pub language: Option<u16>,

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
            cached: row.next_opt::<String>()?
                .map(|v| MetadataItemCached::from_string(&v))
                .unwrap_or_default(),
            isbn_10: row.next()?,
            isbn_13: row.next()?,
            is_public: row.next()?,
            edition_count: row.next::<i64>()? as usize,
            available_at: row.next_opt()?,
            language: row.next::<Option<i16>>()?.map(|v| v as u16),
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
            cached: val.cached,
            isbn_10: val.isbn_10,
            isbn_13: val.isbn_13,
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
            isbn_10: val.isbn_10,
            isbn_13: val.isbn_13,
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

#[allow(clippy::from_over_into)]
impl Into<PublicBook> for BookModel {
    fn into(self) -> PublicBook {
        PublicBook {
            id: *self.id,
            title: self.title,
            clean_title: self.clean_title,
            description: self.description,
            rating: self.rating,
            // We create the thumb_url in the actix request.
            thumb_url: String::new(),
            cached: self.cached,
            isbn_10: self.isbn_10,
            isbn_13: self.isbn_13,
            is_public: self.is_public,
            edition_count: self.edition_count,
            available_at: self.available_at,
            language: self.language,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}


impl BookModel {
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
                    isbn_10, isbn_13,
                    available_at, language,
                    created_at, updated_at, deleted_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) RETURNING id"#,
                params![
                    &self.title, &self.clean_title, &self.description, self.rating, self.thumb_path.as_value(),
                    &self.cached.as_string_optional(), self.is_public, self.edition_count as i64,
                    &self.isbn_10, &self.isbn_13,
                    &self.available_at, self.language.map(|v| v as i16),
                    self.created_at, self.updated_at, self.deleted_at,
                ]
            ).await?;

            self.id = BookId::from(row_int_to_usize(row)?);

            Ok(())
        }
    }

    pub async fn update_book(&mut self, db: &tokio_postgres::Client) -> Result<()> {
        self.updated_at = Utc::now();

        db.execute(r#"
            UPDATE book SET
                title = $2, clean_title = $3, description = $4, rating = $5, thumb_url = $6,
                cached = $7, is_public = $8,
                isbn_10 = $9, isbn_13 = $10,
                available_at = $11, language = $12,
                updated_at = $13, deleted_at = $14
            WHERE id = $1"#,
            params![
                *self.id as i32,
                &self.title, &self.clean_title, &self.description, &self.rating, self.thumb_path.as_value(),
                &self.cached.as_string_optional(), self.is_public,
                &self.isbn_10, &self.isbn_13,
                &self.available_at, &self.language.map(|v| v as i16),
                &self.updated_at, self.deleted_at,
            ]
        ).await?;

        Ok(())
    }

    pub async fn get_by_id(id: BookId, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            "SELECT * FROM book WHERE id = $1",
            params![ *id as i32 ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn exists_by_isbn(value: &str, db: &tokio_postgres::Client) -> Result<bool> {
        Ok(db.query_one(
            "SELECT EXISTS(SELECT id FROM book WHERE isbn_10 = $1 OR isbn_13 = $1)",
            params![ value ],
        ).await?.try_get(0)?)
    }

    pub async fn remove_by_id(id: BookId, db: &tokio_postgres::Client) -> Result<u64> {
        Ok(db.execute(
            "DELETE FROM book WHERE id = $1",
            params![ *id as i32 ]
        ).await?)
    }

    pub async fn get_book_by(offset: usize, limit: usize, _only_public: bool, person_id: Option<PersonId>, db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let inner_query = if let Some(pid) = person_id {
            format!(
                "WHERE id IN (SELECT book_id FROM book_person WHERE person_id = {})",
                pid
            )
        } else {
            String::new()
        };

        let values = db.query(
            &format!("SELECT * FROM book {} LIMIT $1 OFFSET $2", inner_query),
            params![ limit as i64, offset as i64 ]
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }


    fn gen_search_query(query: Option<&str>, only_public: bool, person_id: Option<PersonId>, parameters: &mut Vec<Box<dyn ToSql + Sync>>) -> Option<String> {
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

            // TODO: Seperate from here.
            // Check for possible isbn.
            let isbn = if let Some(isbn) = common::parse_book_id(orig_query).into_possible_isbn_value() {
                format!(" OR isbn_10 = {isbn} OR isbn_13 = {isbn}")
            } else {
                String::new()
            };

            // TODO: Utilize title > clean_title > description, and sort
            sql_queries.push(format!("title LIKE ?? ESCAPE '{escape_char}'{isbn}"));
            parameters.push(Box::new(format!("%{}%", &query)) as Box<dyn ToSql + Sync>);
        }


        // Search with specific person

        if let Some(pid) = person_id {
            sql_queries.push("id IN (SELECT book_id FROM book_person WHERE person_id = ??)".to_string());
            parameters.push(Box::new(*pid as i32) as Box<dyn ToSql + Sync>);
        }

        let sql_query = sql_queries.into_iter()
            .enumerate()
            .map(|(i, v)| v.replace("??", &format!("?{}", base_param_len + 1 + i)))
            .collect::<Vec<_>>()
            .join(" AND ");

        sql += &sql_query;

        if sql.len() == orig_len {
            // If sql is still unmodified
            None
        } else {
            Some(sql)
        }
    }

    pub async fn search_book_list(
        query: Option<&str>,
        offset: usize,
        limit: usize,
        only_public: bool,
        person_id: Option<PersonId>,
        db: &tokio_postgres::Client
    ) -> Result<Vec<Self>> {
        let mut parameters = vec![
            Box::new(limit as i64) as Box<dyn ToSql + Sync>,
            Box::new(offset as i64) as Box<dyn ToSql + Sync>
        ];

        let mut sql = match Self::gen_search_query(query, only_public, person_id, &mut parameters) {
            Some(v) => v,
            None => return Ok(Vec::new())
        };

        sql += " LIMIT $1 OFFSET $2";

        let values = db.query(
            &sql,
            &super::boxed_to_dyn_vec(&parameters)
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn count_search_book(
        query: Option<&str>,
        only_public: bool,
        person_id: Option<PersonId>,
        db: &tokio_postgres::Client
    ) -> Result<usize> {
        let mut parameters = Vec::new();

        let sql = match Self::gen_search_query(query, only_public, person_id, &mut parameters) {
            Some(v) => v.replace("SELECT *", "SELECT COUNT(*)"),
            None => return Ok(0)
        };


        row_bigint_to_usize(db.query_one(&sql, &super::boxed_to_dyn_vec(&parameters)).await?)
    }
}