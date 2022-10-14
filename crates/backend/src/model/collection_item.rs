use common::BookId;
use common_local::CollectionId;

use super::{row_bigint_to_usize, AdvRow, TableRow};
use crate::Result;

pub struct CollectionItemModel {
    pub collection_id: CollectionId,
    pub book_id: BookId,
    pub index: usize,
}

impl TableRow for CollectionItemModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            collection_id: row.next()?,
            book_id: BookId::from(row.next::<i32>()? as usize),
            index: row.next::<i16>()? as usize,
        })
    }
}

impl CollectionItemModel {
    pub async fn find_by_collection_id(
        id: CollectionId,
        db: &tokio_postgres::Client,
    ) -> Result<Vec<Self>> {
        let values = db
            .query(
                "SELECT * FROM collection_item WHERE collection_id = $1",
                params![id],
            )
            .await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn find_by_book_id(id: BookId, db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let values = db
            .query(
                "SELECT * FROM collection_item WHERE book_id = $1",
                params![*id as i32],
            )
            .await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn count_by_collection_id(
        id: CollectionId,
        db: &tokio_postgres::Client,
    ) -> Result<usize> {
        row_bigint_to_usize(
            db.query_one(
                "SELECT COUNT(*) FROM collection_item WHERE collection_id = $1",
                params![id],
            )
            .await?,
        )
    }

    pub async fn insert(&self, db: &tokio_postgres::Client) -> Result<u64> {
        Ok(db
            .execute(
                "INSERT INTO collection_item (collection_id, book_id, idx) VALUES ($1, $2, $3)",
                params![self.collection_id, *self.book_id as i32, self.index as i16,],
            )
            .await?)
    }
}
