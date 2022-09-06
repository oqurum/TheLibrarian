use common::PersonId;
use serde::Serialize;
use tokio_postgres::Client;


use crate::Result;

use super::{TableRow, AdvRow};

#[derive(Debug, Serialize)]
pub struct PersonAltModel {
    pub person_id: PersonId,
    pub name: String,
}

impl TableRow for PersonAltModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            person_id: PersonId::from(row.next::<i64>()? as usize),
            name: row.next()?,
        })
    }
}


impl PersonAltModel {
    pub async fn insert(&self, db: &Client) -> Result<()> {
        db.execute(
            "INSERT INTO person_alt (name, person_id) VALUES (?1, ?2)",
            params![ &self.name, *self.person_id as i64 ]
        ).await?;

        Ok(())
    }

    pub async fn remove(&self, db: &Client) -> Result<u64> {
        Ok(db.execute(
            "DELETE FROM person_alt WHERE name = ?1 AND person_id = ?2",
            params![ &self.name, *self.person_id as i64 ]
        ).await?)
    }


    pub async fn get_by_name(value: &str, db: &Client) -> Result<Option<PersonAltModel>> {
        db.query_opt(
            "SELECT * FROM person_alt WHERE name = ?1",
            params![ value ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn remove_by_person_id(id: PersonId, db: &Client) -> Result<u64> {
        Ok(db.execute(
            "DELETE FROM person_alt WHERE person_id = ?1",
            params![ *id as i64 ]
        ).await?)
    }

    pub async fn transfer_by_person_id(&self, from_id: PersonId, to_id: PersonId, db: &Client) -> Result<u64> {
        Ok(db.execute(
            "UPDATE OR IGNORE person_alt SET person_id = ?2 WHERE person_id = ?1",
            params![ *from_id as i64, *to_id as i64 ]
        ).await?)
    }
}