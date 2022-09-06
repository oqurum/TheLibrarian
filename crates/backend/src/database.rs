use crate::{Result, config::Config};
use tokio_postgres::{connect, NoTls, Client};

pub async fn init(config: &Config) -> Result<Client> {
    let (client, connection) = connect(
        "postgresql://192.168.1.50:5433/librarian?user=postgres&password=postgres",
        NoTls
    ).await?;

    // Initiate Connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            panic!("Database Connection Error: {}", e);
        }
    });


    // Book
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "book" (
            "id"               INTEGER NOT NULL,

            "title"            TEXT,
            "clean_title"      TEXT,
            "description"      TEXT,
            "rating"           FLOAT,
            "thumb_url"        TEXT,

            "cached"           TEXT,

            "isbn_10"          TEXT UNIQUE,
            "isbn_13"          TEXT UNIQUE,

            "is_public"        INTEGER NOT NULL,
            "edition_count"    INTEGER NOT NULL DEFAULT 0,

            "available_at"     DATETIME,
            "language"         INTEGER,

            "created_at"       DATETIME,
            "updated_at"       DATETIME,
            "deleted_at"       DATETIME,

            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;

    // Book People
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "book_person" (
            "book_id"     INTEGER NOT NULL,
            "person_id"   INTEGER NOT NULL,

            UNIQUE(book_id, person_id)
        );"#,
        &[]
    ).await?;

    // People
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "person" (
            "id"            INTEGER NOT NULL,

            "source"        NOT NULL,

            "name"          TEXT NOT NULL COLLATE NOCASE,
            "description"   TEXT,
            "birth_date"    INTEGER,

            "thumb_url"     TEXT,

            "updated_at"    DATETIME NOT NULL,
            "created_at"    DATETIME NOT NULL,

            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;

    // People Other names
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "person_alt" (
            "person_id"   INTEGER NOT NULL,

            "name"        TEXT NOT NULL COLLATE NOCASE,

            UNIQUE(person_id, name)
        );"#,
        &[]
    ).await?;

    // Members
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "members" (
            "id"           INTEGER NOT NULL,

            "name"         TEXT NOT NULL COLLATE NOCASE,
            "email"        TEXT COLLATE NOCASE,
            "password"     TEXT,

            "permissions"  TEXT NOT NULL,

            "created_at"   DATETIME NOT NULL,
            "updated_at"   DATETIME NOT NULL,

            UNIQUE(email),
            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;

    // Auths
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "auths" (
            "oauth_token"          TEXT NOT NULL,
            "oauth_token_secret"   TEXT NOT NULL,

            "created_at"           DATETIME NOT NULL,

            UNIQUE(oauth_token)
        );"#,
        &[]
    ).await?;

    // Tags
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "tags" (
            "id"           INTEGER NOT NULL,

            "name"         TEXT NOT NULL COLLATE NOCASE,
            "type_of"      INTEGER NOT NULL,

            "data"         TEXT,

            "created_at"   DATETIME NOT NULL,
            "updated_at"   DATETIME NOT NULL,

            PRIMARY KEY("id" AUTOINCREMENT),
            UNIQUE("name", "type_of")
        );"#,
        &[]
    ).await?;

    // Book Tags
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "book_tags" (
            "id"          INTEGER NOT NULL,

            "book_id"     INTEGER NOT NULL,
            "tag_id"      INTEGER NOT NULL,

            "windex"      INTEGER NOT NULL,

            "created_at"  DATETIME NOT NULL,

            PRIMARY KEY("id" AUTOINCREMENT),
            UNIQUE("book_id", "tag_id")
        );"#,
        &[]
    ).await?;

    // Uploaded Images
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "uploaded_images" (
            "id"          INTEGER NOT NULL,

            "path"        TEXT NOT NULL,

            "created_at"  DATETIME NOT NULL,

            UNIQUE(path),
            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;

    // Image Link
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "image_link" (
            "image_id"    INTEGER NOT NULL,

            "link_id"     INTEGER NOT NULL,
            "type_of"     INTEGER NOT NULL,

            UNIQUE(image_id, link_id, type_of)
        );"#,
        &[]
    ).await?;


    // Edit
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "edit" (
            "id"           INTEGER NOT NULL,

            "type_of"      INTEGER NOT NULL,
            "operation"    INTEGER NOT NULL,
            "status"       INTEGER NOT NULL,

            "member_id"    INTEGER NOT NULL,
            "model_id"     INTEGER,

            "is_applied"   INTEGER NOT NULL,
            "vote_count"   INTEGER NOT NULL,

            "data"         TEXT NOT NULL,

            "ended_at"     DATETIME,
            "expires_at"   DATETIME,
            "created_at"   DATETIME NOT NULL,
            "updated_at"   DATETIME NOT NULL,

            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;

    // Edit Vote
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "edit_vote" (
            "id"          INTEGER NOT NULL,

            "edit_id"     INTEGER NOT NULL,
            "member_id"   INTEGER NOT NULL,

            "vote"        INTEGER NOT NULL,

            "created_at"  DATETIME NOT NULL,

            UNIQUE("edit_id", "member_id"),
            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;

    // Edit Comment
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "edit_comment" (
            "id"          INTEGER NOT NULL,

            "edit_id"     INTEGER NOT NULL,
            "member_id"   INTEGER NOT NULL,

            "text"        TEXT NOT NULL,
            "deleted"     INTEGER NOT NULL,

            "created_at"  DATETIME NOT NULL,

            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;


    // Linked Servers
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "server_link" (
            "id"                 INTEGER NOT NULL,

            "server_owner_name"  TEXT,
            "server_name"        TEXT,

            "server_id"          TEXT NOT NULL,
            "public_id"          TEXT NOT NULL,

            "member_id"          INTEGER NOT NULL,
            "verified"           INTEGER NOT NULL,

            "created_at"         DATETIME NOT NULL,
            "updated_at"         DATETIME NOT NULL,

            PRIMARY KEY("id" AUTOINCREMENT),
            UNIQUE("server_id")
        );"#,
        &[]
    ).await?;


    // Search Groupings
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "search_group" (
            "id"                INTEGER NOT NULL,

            "query"             TEXT NOT NULL COLLATE NOCASE,
            "calls"             INTEGER NOT NULL,
            "last_found_amount" INTEGER NOT NULL,
            "timeframe"         INTEGER NOT NULL,
            "found_id"          TEXT,

            "created_at"        DATETIME NOT NULL,
            "updated_at"        DATETIME NOT NULL,

            UNIQUE("query", "timeframe"),
            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;

    // Search Server Item
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "search_item" (
            "id"              INTEGER NOT NULL,

            "server_link_id"  INTEGER NOT NULL,

            "query"           TEXT NOT NULL COLLATE NOCASE,
            "calls"           INTEGER NOT NULL,

            "created_at"      DATETIME NOT NULL,
            "updated_at"      DATETIME NOT NULL,

            UNIQUE("query", "server_link_id"),
            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;


    // External Metadata Searches
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "metadata_search" (
            "id"                  INTEGER NOT NULL,

            "query"               TEXT NOT NULL COLLATE NOCASE,
            "agent"               TEXT NOT NULL,
            "type_of"             INTEGER NOT NULL,
            "last_found_amount"   INTEGER NOT NULL,
            "data"                TEXT NOT NULL,

            "created_at"          DATETIME NOT NULL,
            "updated_at"          DATETIME NOT NULL,

            UNIQUE("query", "agent"),
            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;

    // Collection
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "collection" (
            "id"            INTEGER NOT NULL,

            "name"          TEXT NOT NULL COLLATE NOCASE,
            "description"   TEXT,
            "type_of"       INTEGER NOT NULL,

            "created_at"    DATETIME NOT NULL,
            "updated_at"    DATETIME NOT NULL,

            PRIMARY KEY("id" AUTOINCREMENT)
        );"#,
        &[]
    ).await?;


    // Collection Item
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS "collection_item" (
            "collection_id"  INTEGER NOT NULL,
            "book_id"  INTEGER NOT NULL,

            "idx"  INTEGER NOT NULL,

            UNIQUE("collection_id", "book_id")
        );"#,
        &[]
    ).await?;


    // TODO: Tables
    // Queued External Metadata Searches (prevent continuous searching)
    // Fingerprints
    // Custom Book (Fingerprint) Stylings

    Ok(client)
}