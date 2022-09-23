use common::{BookId, PersonId};

use serde::Serialize;

use crate::Result;
use super::{AdvRow, TableRow, BookModel, PersonModel};

#[derive(Debug, Serialize)]
pub struct BookPersonModel {
    pub book_id: BookId,
    pub person_id: PersonId,

    pub info: Option<String>,
}

impl TableRow for BookPersonModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            book_id: BookId::from(row.next::<i32>()? as usize),
            person_id: PersonId::from(row.next::<i32>()? as usize),
            info: row.next_opt()?,
        })
    }
}


impl BookPersonModel {
    pub fn new(book_id: BookId, person_id: PersonId, info: Option<String>) -> Self {
        Self { book_id, person_id, info }
    }

    pub async fn insert(&self, db: &tokio_postgres::Client) -> Result<()> {
        db.execute(
            "INSERT INTO book_person (book_id, person_id, info) VALUES ($1, $2, $3) ON CONFLICT (book_id, person_id) DO NOTHING",
            params![
                *self.book_id as i32,
                *self.person_id as i32,
                self.info.as_deref(),
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

    pub async fn update_book_caches(id: PersonId, person_name: Option<String>, db: &tokio_postgres::Client) -> Result<()> {
        let person_name = if let Some(v) = person_name {
            v
        } else {
            PersonModel::get_by_id(id, db).await?.unwrap().name
        };

        let books = db.query(
            r#"
                SELECT book.*
                FROM book_person
                JOIN book ON book.id = book_person.book_id
                WHERE person_id = $1 AND info = 'Author'
            "#,
            params![*id as i32]
        ).await?.into_iter().map(BookModel::from_row);

        for book in books {
            let mut book = book?;

            book.cached = book.cached.author_id(id).author(person_name.clone());

            db.execute(
                "UPDATE book SET cached = $2 WHERE id = $1",
                params![
                    *book.id as i32,
                    book.cached.as_string()
                ]
            ).await?;
        }

        Ok(())
    }
}