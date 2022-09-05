use common::BookId;
use common_local::CollectionId;
use rusqlite::params;

use crate::{Database, Result};
use super::{AdvRow, TableRow};


pub struct CollectionItemModel {
    pub collection_id: CollectionId,
    pub book_id: BookId,
    pub index: usize,
}


impl TableRow<'_> for CollectionItemModel {
    fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            collection_id: row.next()?,
            book_id: row.next()?,
            index: row.next()?,
        })
    }
}

impl CollectionItemModel {
    pub async fn find_by_collection_id(id: CollectionId, db: &Database) -> Result<Vec<Self>> {
        let this = db.read().await;

        let mut conn = this.prepare("SELECT * FROM collection_item WHERE collection_id = ?1")?;

        let map = conn.query_map([id], |v| Self::from_row(v))?;

        Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub async fn find_by_book_id(id: BookId, db: &Database) -> Result<Vec<Self>> {
        let this = db.read().await;

        let mut conn = this.prepare("SELECT * FROM collection_item WHERE book_id = ?1")?;

        let map = conn.query_map([id], |v| Self::from_row(v))?;

        Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub async fn count_by_collection_id(id: CollectionId, db: &Database) -> Result<usize> {
        Ok(db.read().await.query_row(
            "SELECT COUNT(*) FROM collection_item WHERE collection_id = ?1",
            [id],
            |v| v.get(0)
        )?)
    }

    pub async fn insert(&self, db: &Database) -> Result<usize> {
        Ok(db.write().await.execute(
            "INSERT INTO collection_item (collection_id, book_id, idx) VALUES (?1, ?2, ?3)",
            params![
                self.collection_id,
                self.book_id,
                self.index,
            ]
        )?)
    }
}