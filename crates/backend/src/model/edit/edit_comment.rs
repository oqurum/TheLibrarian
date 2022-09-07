use chrono::{DateTime, Utc};
use common::MemberId;
use common_local::{EditId, EditCommentId};
use tokio_postgres::Client;


use crate::{Result, model::{TableRow, AdvRow, row_int_to_usize, row_bigint_to_usize}};



pub struct NewEditCommentModel {
    pub edit_id: EditId,
    pub member_id: MemberId,

    pub text: String,
    pub deleted: bool,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EditCommentModel {
    pub id: EditCommentId,

    pub edit_id: EditId,
    pub member_id: MemberId,

    pub text: String,
    pub deleted: bool,

    pub created_at: DateTime<Utc>,
}


impl TableRow for EditCommentModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: row.next()?,

            edit_id: row.next()?,
            member_id: MemberId::from(row.next::<i32>()? as usize),

            text: row.next()?,
            deleted: row.next()?,

            created_at: row.next()?,
        })
    }
}


impl NewEditCommentModel {
    pub fn new(edit_id: EditId, member_id: MemberId, text: String) -> Self {
        Self {
            edit_id,
            member_id,
            text,
            deleted: false,
            created_at: Utc::now(),
        }
    }

    pub async fn insert(self, client: &Client) -> Result<EditCommentModel> {
        let row = client.query_one(r#"
            INSERT INTO edit_comment (
                edit_id, member_id,
                text, deleted,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5) RETURNING id"#,
            params![
                self.edit_id, *self.member_id as i64,
                self.text, self.deleted,
                self.created_at,
            ]
        ).await?;

        Ok(EditCommentModel {
            id: EditCommentId::from(row_int_to_usize(row)?),

            edit_id: self.edit_id,
            member_id: self.member_id,

            text: self.text,
            deleted: self.deleted,

            created_at: self.created_at,
        })
    }
}


impl EditCommentModel {
    pub async fn get_by_edit_id(
        edit_id: EditId,
        offset: usize,
        limit: usize,
        deleted: Option<bool>,
        client: &Client
    ) -> Result<Vec<Self>> {
        if let Some(deleted) = deleted {
            let conn = client.query(
                "SELECT * FROM edit_comment WHERE edit_id = $1 AND deleted = $2 LIMIT $3 OFFSET $4",
                params![ edit_id, deleted, limit as i64, offset as i64 ]
            ).await?;

            Ok(conn.into_iter().map(Self::from_row).collect::<std::result::Result<Vec<_>, _>>()?)
        } else {
            let conn = client.query(
                "SELECT * FROM edit_comment WHERE edit_id = $1 LIMIT $2 OFFSET $3",
                params![ edit_id, limit as i64, offset as i64 ]
            ).await?;

            Ok(conn.into_iter().map(Self::from_row).collect::<std::result::Result<Vec<_>, _>>()?)
        }
    }

    pub async fn get_count(edit_id: EditId, client: &Client) -> Result<usize> {
        row_bigint_to_usize(client.query_one(r#"SELECT COUNT(*) FROM edit_comment WHERE edit_id = $1"#, params![ edit_id ]).await?)
    }
}