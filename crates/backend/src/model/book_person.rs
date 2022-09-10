use common::{BookId, PersonId};

use serde::Serialize;

use crate::Result;
use super::{AdvRow, TableRow};

#[derive(Debug, Serialize)]
pub struct BookPersonModel {
    pub book_id: BookId,
    pub person_id: PersonId,
}

impl TableRow for BookPersonModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            book_id: BookId::from(row.next::<i32>()? as usize),
            person_id: PersonId::from(row.next::<i32>()? as usize),
        })
    }
}


impl BookPersonModel {
    pub fn new(book_id: BookId, person_id: PersonId) -> Self {
        Self { book_id, person_id }
    }

    pub async fn insert(&self, db: &tokio_postgres::Client) -> Result<()> {
        db.execute(
            "INSERT INTO book_person (book_id, person_id) VALUES ($1, $2) ON CONFLICT (book_id, person_id) DO NOTHING",
            params![
                *self.book_id as i32,
                *self.person_id as i32
            ]
        ).await?;

        Ok(())
    }

    pub async fn remove(&self, db: &tokio_postgres::Client) -> Result<()> {
        db.execute(
            "DELETE FROM book_person WHERE book_id = $1 AND person_id = $2",
            params![
                *self.book_id as i32,
                *self.person_id as i32
            ]
        ).await?;

        Ok(())
    }

    pub async fn remove_by_book_id(id: BookId, db: &tokio_postgres::Client) -> Result<()> {
        db.execute(
            "DELETE FROM book_person WHERE book_id = $1",
            params![ *id as i32 ]
        ).await?;

        Ok(())
    }

    pub async fn remove_by_person_id(id: PersonId, db: &tokio_postgres::Client) -> Result<()> {
        db.execute("DELETE FROM book_person WHERE person_id = $1",
            params![ *id as i32 ]
        ).await?;

        Ok(())
    }

    pub async fn transfer(from_id: PersonId, to_id: PersonId, db: &tokio_postgres::Client) -> Result<u64> {
        Ok(db.execute("UPDATE book_person SET person_id = $2 WHERE person_id = $1",
            params![ *from_id as i32, *to_id as i32 ]
        ).await?)
    }

    pub async fn get_all_by_book_id(book_id: BookId, db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let conn = db.query(
            "SELECT * FROM book_person WHERE book_id = $1",
            params![ *book_id as i32 ]
        ).await?;

        conn.into_iter().map(Self::from_row).collect()
    }

    pub async fn find_by_person_id(id: PersonId, db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let conn = db.query(
            "SELECT * FROM book_person WHERE person_id = $1",
            params![ *id as i32 ]
        ).await?;

        conn.into_iter().map(Self::from_row).collect()
    }
}