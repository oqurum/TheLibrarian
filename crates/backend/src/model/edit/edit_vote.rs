use chrono::{DateTime, Utc};
use common::MemberId;
use common_local::{EditId, item::edit::SharedEditVoteModel, EditVoteId};
use tokio_postgres::Client;

use crate::{Result, model::{TableRow, AdvRow, row_to_usize}};



#[derive(Debug, Clone)]
pub struct EditVoteModel {
    pub id: EditVoteId,

    pub edit_id: EditId,
    pub member_id: MemberId,

    pub vote: bool,

    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Clone)]
pub struct NewEditVoteModel {
    pub edit_id: EditId,
    pub member_id: MemberId,

    pub vote: bool,

    pub created_at: DateTime<Utc>,
}


impl TableRow for EditVoteModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: row.next()?,
            edit_id: row.next()?,
            member_id: MemberId::from(row.next::<i64>()? as usize),

            vote: row.next()?,

            created_at: row.next()?,
        })
    }
}


impl From<EditVoteModel> for SharedEditVoteModel {
    fn from(val: EditVoteModel) -> Self {
        SharedEditVoteModel {
            id: val.id,
            edit_id: val.edit_id,
            member_id: Some(val.member_id),
            vote: val.vote,
            created_at: val.created_at,
        }
    }
}


impl NewEditVoteModel {
    pub fn create(edit_id: EditId, member_id: MemberId, vote: bool) -> Self {
        Self {
            edit_id,
            member_id,
            vote,
            created_at: Utc::now(),
        }
    }

    pub async fn insert(self, client: &Client) -> Result<EditVoteModel> {
        let row = client.query_one(
            "INSERT INTO edit_vote (edit_id, member_id, vote, created_at) VALUES ($1, $2, $3, $4) RETURNING id",
            params![
                self.edit_id,
                *self.member_id as i64,
                self.vote,
                self.created_at,
            ]
        ).await?;

        Ok(EditVoteModel {
            id: EditVoteId::from(row_to_usize(row)?),
            edit_id: self.edit_id,
            member_id: self.member_id,
            vote: self.vote,
            created_at: self.created_at,
        })
    }
}

impl EditVoteModel {
    pub async fn update(&self, client: &Client) -> Result<()> {
        Self::update_vote(self.edit_id, self.member_id, self.vote, client).await
    }


    pub async fn update_vote(edit_id: EditId, member_id: MemberId, vote: bool, client: &Client) -> Result<()> {
        client.execute(
            "UPDATE edit_vote SET vote = $3 WHERE edit_id = $1 AND member_id = $2",
            params![
                edit_id,
                *member_id as i64,
                vote,
            ]
        ).await?;

        Ok(())
    }

    pub async fn remove(edit_id: EditId, member_id: MemberId, client: &Client) -> Result<u64> {
        Ok(client.execute(
            "DELETE FROM edit_vote WHERE edit_id = $1 AND member_id = $2",
            params![
                edit_id,
                *member_id as i64,
            ]
        ).await?)
    }

    pub async fn find_one(edit_id: EditId, member_id: MemberId, client: &Client) -> Result<Option<Self>> {
        client.query_opt(
            "SELECT * FROM edit_vote WHERE edit_id = $1 AND member_id = $2",
            params![
                edit_id,
                *member_id as i64,
            ]
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn find_by_edit_id(edit_id: EditId, offset: usize, limit: usize, client: &Client) -> Result<Vec<Self>> {
        let conn = client.query(
            "SELECT * FROM edit_vote WHERE edit_id = $1 LIMIT $2 OFFSET $3",
            params![edit_id, limit as i64, offset as i64]
        ).await?;

        conn.into_iter().map(Self::from_row).collect()
    }

    pub async fn count_by_edit_id(edit_id: EditId, client: &Client) -> Result<usize> {
        row_to_usize(client.query_one("SELECT COUNT(*) FROM edit_vote WHERE edit_id = $1", params![edit_id]).await?)
    }
}