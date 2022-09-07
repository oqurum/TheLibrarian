use crate::{Result, config::Config};
use tokio_postgres::{connect, NoTls, Client};

pub async fn init(config: &Config) -> Result<Client> {
    let (client, connection) = connect(
        &config.database.url,
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
        r#"CREATE TABLE IF NOT EXISTS book (
            id               SERIAL PRIMARY KEY,

            title            TEXT,
            clean_title      TEXT,
            description      TEXT,
            rating           DOUBLE PRECISION,
            thumb_url        CHAR(64),

            cached           TEXT,

            isbn_10          CHAR(10) UNIQUE,
            isbn_13          CHAR(13) UNIQUE,

            is_public        BOOL NOT NULL,
            edition_count    BIGINT NOT NULL,

            available_at     DATE,
            language         SMALLINT,

            created_at       TIMESTAMPTZ,
            updated_at       TIMESTAMPTZ,
            deleted_at       TIMESTAMPTZ
        );"#,
        &[]
    ).await?;

    // People
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS person (
            id            SERIAL PRIMARY KEY,

            source        TEXT NOT NULL,

            name          TEXT NOT NULL,
            description   TEXT,
            birth_date    DATE,

            thumb_url     CHAR(64),

            updated_at    TIMESTAMPTZ NOT NULL,
            created_at    TIMESTAMPTZ NOT NULL
        );"#,
        &[]
    ).await?;

    // People Other names
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS person_alt (
            person_id   INT references person(id) ON DELETE CASCADE,

            name        TEXT NOT NULL,

            UNIQUE(person_id, name)
        );"#,
        &[]
    ).await?;

    // Book People
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS book_person (
            book_id     INT NOT NULL references book(id) ON DELETE CASCADE,
            person_id   INT NOT NULL references person(id) ON DELETE CASCADE,

            UNIQUE(book_id, person_id)
        );"#,
        &[]
    ).await?;

    // Members
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS member (
            id           SERIAL PRIMARY KEY,

            name         TEXT NOT NULL,
            email        VARCHAR(32),
            password     VARCHAR(128),

            permissions  TEXT NOT NULL,

            created_at   TIMESTAMPTZ NOT NULL,
            updated_at   TIMESTAMPTZ NOT NULL,

            UNIQUE(email)
        );"#,
        &[]
    ).await?;

    // Auths
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS auth (
            oauth_token          TEXT NOT NULL,
            oauth_token_secret   TEXT NOT NULL,

            created_at           TIMESTAMPTZ NOT NULL,

            UNIQUE(oauth_token)
        );"#,
        &[]
    ).await?;

    // Tags
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS tag (
            id           SERIAL PRIMARY KEY,

            name         VARCHAR(32) NOT NULL,
            type_of      SMALLINT NOT NULL,

            data         TEXT,

            created_at   TIMESTAMPTZ NOT NULL,
            updated_at   TIMESTAMPTZ NOT NULL,

            UNIQUE("name", "type_of")
        );"#,
        &[]
    ).await?;

    // Book Tags
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS book_tag (
            id          SERIAL PRIMARY KEY,

            book_id     INT NOT NULL references book(id) ON DELETE CASCADE,
            tag_id      INT NOT NULL references tag(id) ON DELETE CASCADE,

            idx         SMALLINT NOT NULL,

            created_at  TIMESTAMPTZ NOT NULL,

            UNIQUE("book_id", "tag_id")
        );"#,
        &[]
    ).await?;

    // Uploaded Images
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS uploaded_image (
            id          SERIAL PRIMARY KEY,

            path        TEXT NOT NULL,

            created_at  TIMESTAMPTZ NOT NULL,

            UNIQUE(path)
        );"#,
        &[]
    ).await?;

    // Image Link
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS image_link (
            "image_id"    INT NOT NULL references uploaded_image(id) ON DELETE CASCADE,

            "link_id"     INT NOT NULL,
            "type_of"     SMALLINT NOT NULL,

            UNIQUE(image_id, link_id, type_of)
        );"#,
        &[]
    ).await?;


    // Edit
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS edit (
            id           SERIAL PRIMARY KEY,

            type_of      SMALLINT NOT NULL,
            operation    SMALLINT NOT NULL,
            status       SMALLINT NOT NULL,

            member_id    INT NOT NULL references member(id) ON DELETE CASCADE,
            model_id     INT,

            is_applied   BOOL NOT NULL,
            vote_count   SMALLINT NOT NULL,

            data         TEXT NOT NULL,

            ended_at     TIMESTAMPTZ,
            expires_at   TIMESTAMPTZ,
            created_at   TIMESTAMPTZ NOT NULL,
            updated_at   TIMESTAMPTZ NOT NULL
        );"#,
        &[]
    ).await?;

    // Edit Vote
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS edit_vote (
            id          SERIAL PRIMARY KEY,

            edit_id     INT NOT NULL references edit(id) ON DELETE CASCADE,
            member_id   INT NOT NULL references member(id) ON DELETE CASCADE,

            vote        BOOL NOT NULL,

            created_at  TIMESTAMPTZ NOT NULL,

            UNIQUE("edit_id", "member_id")
        );"#,
        &[]
    ).await?;

    // Edit Comment
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS edit_comment (
            id          SERIAL PRIMARY KEY,

            edit_id     INT NOT NULL references edit(id) ON DELETE CASCADE,
            member_id   INT NOT NULL references member(id) ON DELETE CASCADE,

            text        TEXT NOT NULL,
            deleted     BOOL NOT NULL,

            created_at  TIMESTAMPTZ NOT NULL

        );"#,
        &[]
    ).await?;


    // Linked Servers
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS server_link (
            id                 SERIAL PRIMARY KEY,

            server_owner_name  VARCHAR(32),
            server_name        VARCHAR(32),

            server_id          TEXT NOT NULL,
            public_id          TEXT NOT NULL,

            member_id          INT NOT NULL references member(id) ON DELETE CASCADE,
            verified           BOOL NOT NULL,

            created_at         TIMESTAMPTZ NOT NULL,
            updated_at         TIMESTAMPTZ NOT NULL,

            UNIQUE("server_id")
        );"#,
        &[]
    ).await?;


    // Search Groupings
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS search_group (
            id                SERIAL PRIMARY KEY,

            query             TEXT NOT NULL,
            calls             INT NOT NULL,
            last_found_amount SMALLINT NOT NULL,
            timeframe         INT NOT NULL,
            found_id          TEXT,

            created_at        TIMESTAMPTZ NOT NULL,
            updated_at        TIMESTAMPTZ NOT NULL,

            UNIQUE("query", "timeframe")
        );"#,
        &[]
    ).await?;

    // Search Server Item
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS search_item (
            id              SERIAL PRIMARY KEY,

            server_link_id  INT NOT NULL references server_link(id) ON DELETE CASCADE,

            query           TEXT NOT NULL,
            calls           INT NOT NULL,

            created_at      TIMESTAMPTZ NOT NULL,
            updated_at      TIMESTAMPTZ NOT NULL,

            UNIQUE("query", "server_link_id")
        );"#,
        &[]
    ).await?;


    // External Metadata Searches
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS metadata_search (
            id                  SERIAL PRIMARY KEY,

            query               TEXT NOT NULL,
            agent               VARCHAR(16) NOT NULL,
            type_of             SMALLINT NOT NULL,
            last_found_amount   INT NOT NULL,
            data                TEXT NOT NULL,

            created_at          TIMESTAMPTZ NOT NULL,
            updated_at          TIMESTAMPTZ NOT NULL,

            UNIQUE("query", "agent")
        );"#,
        &[]
    ).await?;

    // Collection
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS collection (
            id            SERIAL PRIMARY KEY,

            name          TEXT NOT NULL,
            description   TEXT,
            type_of       SMALLINT NOT NULL,

            created_at    TIMESTAMPTZ NOT NULL,
            updated_at    TIMESTAMPTZ NOT NULL
        );"#,
        &[]
    ).await?;


    // Collection Item
    client.execute(
        r#"CREATE TABLE IF NOT EXISTS collection_item (
            collection_id  INT NOT NULL references collection(id) ON DELETE CASCADE,
            book_id        INT NOT NULL references book(id) ON DELETE CASCADE,

            idx            SMALLINT NOT NULL,

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