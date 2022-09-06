use common::{BookId, TagId, BookTagId};
use chrono::{DateTime, Utc};
use common_local::BookTag;
use tokio_postgres::Client;


use crate::Result;

use super::{TagModel, AdvRow, TableRow, row_to_usize};

pub struct BookTagModel {
    pub id: BookTagId,

    pub book_id: BookId,
    pub tag_id: TagId,

    pub index: usize,

    pub created_at: DateTime<Utc>,
}

impl TableRow for BookTagModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: BookTagId::from(row.next::<i64>()? as usize),

            book_id: BookId::from(row.next::<i64>()? as usize),
            tag_id: TagId::from(row.next::<i64>()? as usize),

            index: row.next::<i64>()? as usize,

            created_at: row.next()?,
        })
    }
}



pub struct BookTagWithTagModel {
    pub id: BookTagId,

    pub book_id: BookId,

    pub index: usize,

    pub created_at: DateTime<Utc>,

    pub tag: TagModel,
}

impl TableRow for BookTagWithTagModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: BookTagId::from(row.next::<i64>()? as usize),

            book_id: BookId::from(row.next::<i64>()? as usize),
            index: row.next::<i64>()? as usize,

            created_at: row.next()?,

            tag: TagModel::create(row)?
        })
    }
}

impl From<BookTagWithTagModel> for BookTag {
    fn from(val: BookTagWithTagModel) -> Self {
        BookTag {
            id: val.id,
            book_id: val.book_id,
            index: val.index,
            created_at: val.created_at,
            tag: val.tag.into(),
        }
    }
}



impl BookTagModel {
    pub async fn get_by_id(id: BookTagId, db: &Client) -> Result<Option<Self>> {
        db.query_opt(
            "SELECT * FROM book_tag WHERE id = $1",
            params![ *id as i64 ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn remove(book_id: BookId, tag_id: TagId, db: &Client) -> Result<u64> {
        Ok(db.execute(
            "DELETE FROM book_tag WHERE book_id = $1 AND tag_id = $2",
            params![ *book_id as i64, *tag_id as i64 ],
        ).await?)
    }

    pub async fn insert(book_id: BookId, tag_id: TagId, index: Option<usize>, db: &Client) -> Result<Self> {
        let index = if let Some(index) = index {
            db.execute(
                "UPDATE book_tag SET idx = idx + 1 WHERE book_id = $1 AND tag_id = $2 AND idx >= $3",
                params![ *book_id as i64, *tag_id as i64, index as i64 ],
            ).await?;

            index
        } else {
            Self::count_book_tags_by_bid_tid(book_id, tag_id, db).await?
        };

        let created_at = Utc::now();

        let row = db.query_one(
            "INSERT INTO book_tag (book_id, tag_id, idx, created_at) VALUES ($1, $2, $3, $4) RETURNING id",
            params![
                *book_id as i64,
                *tag_id as i64,
                index as i64,
                created_at,
            ]
        ).await?;

        Ok(Self {
            id: BookTagId::from(row_to_usize(row)?),
            book_id,
            tag_id,
            index,
            created_at,
        })
    }

    pub async fn count_book_tags_by_bid_tid(book_id: BookId, tag_id: TagId, db: &Client) -> Result<usize> {
        row_to_usize(db.query_one(
            "SELECT COUNT(*) FROM book_tag WHERE book_id = $1 AND tag_id = $2",
            params![ *book_id as i64, *tag_id as i64 ],
        ).await?)
    }

    pub async fn get_books_by_book_id(book_id: BookId, db: &Client) -> Result<Vec<Self>> {
        let conn = db.query(
            "SELECT * FROM book_tag WHERE book_id = $1",
            params![ *book_id as i64 ]
        ).await?;

        conn.into_iter().map(Self::from_row).collect()
    }
}


impl BookTagWithTagModel {
    pub async fn get_by_book_id(book_id: BookId, db: &Client) -> Result<Vec<Self>> {
        let conn = db.query(
            r#"SELECT book_tag.id, book_tag.book_id, idx, book_tag.created_at, tags.*
            FROM book_tag
            JOIN tags ON book_tag.tag_id == tags.id
            WHERE book_id = $1"#,
            params![ *book_id as i64 ]
        ).await?;

        conn.into_iter().map(Self::from_row).collect()
    }

    pub async fn get_by_book_id_and_tag_id(book_id: BookId, tag_id: TagId, db: &Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT book_tag.id, book_tag.book_id, idx, book_tag.created_at, tags.*
            FROM book_tag
            JOIN tags ON book_tag.tag_id == tags.id
            WHERE book_id = $1 AND tag_id = $2"#,
            params![ *book_id as i64, *tag_id as i64 ],
        ).await?.map(Self::from_row).transpose()
    }
}