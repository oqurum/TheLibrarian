use common::BookId;
use tokio_postgres::Client;

use crate::Result;

use super::{AdvRow, TableRow};

pub struct BookIsbnModel {
    pub isbn: String,

    pub book_id: BookId,
}

impl TableRow for BookIsbnModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            isbn: row.next()?,

            book_id: BookId::from(row.next::<i32>()? as usize),
        })
    }
}

impl BookIsbnModel {
    pub async fn get_by_id(isbn: &str, db: &Client) -> Result<bool> {
        Ok(db.query_one("SELECT EXISTS(SELECT * FROM book_isbn WHERE isbn = $1)", params![isbn])
            .await?
            .try_get::<_, bool>(0)?)
    }

    pub async fn get_all(id: BookId, db: &Client) -> Result<Vec<Self>> {
        let conn = db
            .query(
                "SELECT * FROM book_isbn WHERE book_id = $1",
                params![*id as i32],
            )
            .await?;

        conn.into_iter().map(Self::from_row).collect()
    }

    pub async fn remove_isbn(isbn: &str, db: &Client) -> Result<u64> {
        Ok(db
            .execute(
                "DELETE FROM book_isbn WHERE isbn = $1",
                params![isbn],
            )
            .await?)
    }

    pub async fn insert(
        &self,
        db: &Client,
    ) -> Result<u64> {
        Ok(db.execute(
            "INSERT INTO book_isbn (isbn, book_id) VALUES ($1, $2)",
            params![
                &self.isbn,
                *self.book_id as i32,
            ]
        ).await?)
    }
}