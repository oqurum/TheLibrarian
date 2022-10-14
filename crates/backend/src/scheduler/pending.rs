use chrono::Utc;
use common_local::edit::*;
use tokio_postgres::Client;

use crate::{
    model::{EditModel, NewEditCommentModel, TableRow, SYSTEM_MEMBER_ID},
    Result,
};

pub async fn task_update_pending(client: &Client) -> Result<()> {
    let now = Utc::now();
    let pending = u8::from(EditStatus::Pending);

    let sql_rejected = "UPDATE edit SET status = $1, ended_at = $2, is_applied = true WHERE expires_at < $2 AND status = $3 AND vote_count < 0";

    {
        // Get all rejected
        let items = {
            let conn = client
                .query(
                    sql_rejected,
                    params![EditStatus::Rejected, now, pending as i16],
                )
                .await?;

            conn.into_iter()
                .map(EditModel::from_row)
                .collect::<Result<Vec<_>>>()?
        };

        for item in items {
            NewEditCommentModel::new(
                item.id,
                *SYSTEM_MEMBER_ID,
                String::from(r#"SYSTEM: Auto denied."#),
            )
            .insert(client)
            .await?;
        }
    }

    // Reject All
    client
        .execute(
            sql_rejected,
            params![EditStatus::Rejected, now, pending as i16],
        )
        .await?;

    {
        // Get all approved
        let items =
            {
                let conn = client.query(
                "SELECT * FROM edit WHERE expires_at < $1 AND status = $2 AND vote_count >= 0",
                params![ now, pending as i16 ],
            ).await?;

                conn.into_iter()
                    .map(EditModel::from_row)
                    .collect::<Result<Vec<_>>>()?
            };

        for mut item in items {
            item.process_status_change(EditStatus::Accepted, client)
                .await?;

            NewEditCommentModel::new(
                item.id,
                *SYSTEM_MEMBER_ID,
                String::from(r#"SYSTEM: Auto accepted."#),
            )
            .insert(client)
            .await?;
        }
    }

    Ok(())
}
