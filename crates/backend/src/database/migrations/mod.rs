use chrono::{DateTime, Utc};
use tokio_postgres::Client;

use crate::Result;

mod main;

pub async fn start_initiation(client: &Client) -> Result<()> {
    if does_migration_table_exist(client).await? {
        // TODO: Handle Migrations
    } else {
        main::init(client).await?;
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

    name: String,
    notes: String,

    created_at: DateTime<Utc>,
}

impl MigrationModel {
    //
}
