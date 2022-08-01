use chrono::Utc;
use common_local::edit::*;
use rusqlite::params;

use crate::{Database, Result, model::{NewEditCommentModel, EditModel, SYSTEM_MEMBER, TableRow}};



pub async fn task_update_pending(db: &Database) -> Result<()> {
	let now = Utc::now().timestamp_millis();
	let pending = u8::from(EditStatus::Pending);

	let sql_rejected = "UPDATE edit SET status = ?1, ended_at = ?2, is_applied = 1 WHERE expires_at < ?2 AND status = ?3 AND vote_count < 0";

	{ // Get all rejected
		let items = {
			let this = db.read().await;

			let mut conn = this.prepare(sql_rejected)?;

			let map = conn.query_map(
				params![ EditStatus::Rejected, now, pending ],
				|v| EditModel::from_row(v)
			)?;

			map.collect::<std::result::Result<Vec<_>, _>>()?
		};

		for item in items {
			NewEditCommentModel::new(
				item.id,
				SYSTEM_MEMBER.id,
				String::from(r#"SYSTEM: Auto denied."#)
			).insert(db).await?;
		}
	}

	// Reject All
	db.write().await
	.execute(sql_rejected, params![ EditStatus::Rejected, now, pending ])?;


	{ // Get all approved
		let items = {
			let this = db.read().await;

			let mut conn = this.prepare("SELECT * FROM edit WHERE expires_at < ?1 AND status = ?2 AND vote_count >= 0")?;

			let map = conn.query_map(params![ now, pending ], |v| EditModel::from_row(v))?;

			map.collect::<std::result::Result<Vec<_>, _>>()?
		};

		for mut item in items {
			println!("{:#?}", item);
			item.process_status_change(EditStatus::Accepted, db).await?;

			NewEditCommentModel::new(
				item.id,
				SYSTEM_MEMBER.id,
				String::from(r#"SYSTEM: Auto accepted."#)
			).insert(db).await?;
		}
	}

	Ok(())
}