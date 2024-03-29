use common::PersonId;
use serde::Serialize;
use tokio_postgres::Client;

use crate::Result;

use super::{AdvRow, TableRow};

#[derive(Debug, Serialize)]
pub struct PersonAltModel {
    pub person_id: PersonId,
    pub name: String,
}

impl TableRow for PersonAltModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            person_id: PersonId::from(row.next::<i32>()? as usize),
            name: row.next()?,
        })
    }
}

impl PersonAltModel {
    pub async fn insert(&self, db: &Client) -> Result<()> {
        db.execute(
            "INSERT INTO person_alt (name, person_id) VALUES ($1, $2)",
            params![&self.name, *self.person_id as i32],
        )
        .await?;

        Ok(())
    }

    pub async fn remove(&self, db: &Client) -> Result<u64> {
        Ok(db
            .execute(
                "DELETE FROM person_alt WHERE name = $1 AND person_id = $2",
                params![&self.name, *self.person_id as i32],
            )
            .await?)
    }

    pub async fn find_all_by_person_id(id: PersonId, db: &Client) -> Result<Vec<Self>> {
        let query = db
            .query(
                "SELECT * FROM person_alt WHERE person_id = $1",
                params![*id as i32],
            )
            .await?;

        query.into_iter().map(Self::from_row).collect()
    }

    pub async fn get_by_name(value: &str, db: &Client) -> Result<Option<Self>> {
        db.query_opt("SELECT * FROM person_alt WHERE name = $1", params![value])
            .await?
            .map(Self::from_row)
            .transpose()
    }

    pub async fn remove_by_person_id(id: PersonId, db: &Client) -> Result<u64> {
        Ok(db
            .execute(
                "DELETE FROM person_alt WHERE person_id = $1",
                params![*id as i32],
            )
            .await?)
    }

    pub async fn transfer_by_person_id(
        from_id: PersonId,
        to_id: PersonId,
        db: &Client,
    ) -> Result<u64> {
        Ok(db
            .execute(
                "UPDATE person_alt SET person_id = $2 WHERE person_id = $1",
                params![*from_id as i32, *to_id as i32],
            )
            .await?)
    }
}
