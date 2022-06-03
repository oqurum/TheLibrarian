use std::{thread, time::Duration};

use crate::{Result, Database, model::EditModel};



pub fn start(db: Database) -> thread::JoinHandle<Result<()>> {
	thread::spawn(move || {
		let rt = tokio::runtime::Runtime::new().unwrap();

		rt.block_on(async {
			loop {
				if let Err(e) = EditModel::task_update_pending(&db).await {
					eprintln!("{}", e);
				}

				tokio::time::sleep(Duration::from_secs(60 * 15)).await;
			}
		})
	})
}