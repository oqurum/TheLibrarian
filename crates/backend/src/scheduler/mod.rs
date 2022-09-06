use std::{thread, time::Duration};

use crate::Result;

mod pending;

pub fn start(db: actix_web::web::Data<tokio_postgres::Client>) -> thread::JoinHandle<Result<()>> {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            loop {
                if let Err(e) = pending::task_update_pending(&*db).await {
                    eprintln!("{}", e);
                }

                tokio::time::sleep(Duration::from_secs(60 * 15)).await;
            }
        })
    })
}