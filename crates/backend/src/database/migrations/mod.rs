use std::time::Instant;

use chrono::{DateTime, Utc};
use tokio_postgres::Client;

use crate::Result;

mod main;

const MIGRATIONS: [(i32, &str, &str, &str); 1] = [
    (1, "1_isbn_separation", include_str!("files/1_isbn_separation.sql"), "Separate ISBN's into own table")
];


pub async fn start_initiation(db: &mut Client) -> Result<()> {
    if does_migration_table_exist(db).await? {
        let items = MigrationModel::get_all(db).await?;
        let last_index = items.into_iter().fold(0, |idx, b| idx.max(b.id));

        for (id, name, sql, notes) in MIGRATIONS {
            if id > last_index {
                let now = Instant::now();

                let trx = db.transaction().await?;

                trx.batch_execute(sql).await?;

                trx.commit().await?;

                MigrationModel {
                    id,
                    duration: now.elapsed().as_millis() as i32,
                    title: name.to_string(),
                    notes: notes.to_string(),
                    created_at: Utc::now(),
                }.insert(db).await?;
            }
        }

        // TODO: Handle Migrations
    } else {
        // Fill Migrations with filler.
        for (id, name, _sql, notes) in MIGRATIONS {
            MigrationModel {
                id,
                duration: 0,
                title: name.to_string(),
                notes: notes.to_string(),
                created_at: Utc::now(),
            }.insert(db).await?;
        }

        main::init(db).await?;
    }

    Ok(())
}

async fn does_migration_table_exist(client: &Client) -> Result<bool> {
    Ok(client
        .query_one(
            r#"SELECT EXISTS (
            SELECT FROM
                pg_tables
            WHERE
                schemaname = 'public' AND
                tablename  = 'migration'
        );"#,
            params![],
        )
        .await?
        .get(0))
}

struct MigrationModel {
    id: i32,

    duration: i32,

    title: String,
    notes: String,

    created_at: DateTime<Utc>,
}

impl MigrationModel {
    pub async fn insert(
        &mut self,
        db: &Client,
    ) -> Result<u64> {
        Ok(db.execute(
            "INSERT INTO migration (id, title, duration, notes, created_at) VALUES ($1, $2, $3, $4, $5)",
            params![
                self.id,
                &self.title,
                self.duration,
                &self.notes,
                self.created_at
            ]
        ).await?)
    }

    pub async fn get_all(db: &Client) -> Result<Vec<Self>> {
        let conn = db
            .query("SELECT * FROM migration", &[])
            .await?;

        conn.into_iter().map(|row| Ok(Self {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            duration: row.try_get(2)?,
            notes: row.try_get(3)?,
            created_at: row.try_get(4)?,
        })).collect()
    }
}
