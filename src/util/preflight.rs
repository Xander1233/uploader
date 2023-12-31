use tokio_postgres::Client;

pub async fn preflight(client: &Client) {
    client
        .query(
            "CREATE TABLE IF NOT EXISTS users ( \
        id text NOT NULL PRIMARY KEY, \
        password text NOT NULL, \
        username text UNIQUE NOT NULL, \
        permission_level int NOT NULL DEFAULT 0, \
        display_name text NOT NULL, \
        email text NOT NULL, \
        created_at TIMESTAMP NOT NULL DEFAULT NOW() \
    )",
            &[],
        )
        .await
        .unwrap();

    client
        .query(
            "CREATE TABLE IF NOT EXISTS sessions ( \
        id text NOT NULL PRIMARY KEY, \
        auth text NOT NULL, \
        userid text, \
        created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
        CONSTRAINT fk_id \
            FOREIGN KEY (userid) \
                REFERENCES users(id) ON DELETE CASCADE \
    )",
            &[],
        )
        .await
        .unwrap();

    client
        .query(
            "CREATE TABLE IF NOT EXISTS files ( \
        id text PRIMARY KEY, \
        userid text, \
        data bytea,\
        FOREIGN KEY (userid) REFERENCES users (id) ON DELETE CASCADE \
    )",
            &[],
        )
        .await
        .unwrap();

    client
        .query(
            "CREATE TABLE IF NOT EXISTS metadata ( \
        id text PRIMARY KEY, \
        userid text, \
        filetype text, \
        is_private boolean, \
        password text DEFAULT '', \
        uploaded_at TIMESTAMP NOT NULL DEFAULT NOW(), \
        CONSTRAINT fk_id \
            FOREIGN KEY (id) \
                REFERENCES files(id) ON DELETE CASCADE, \
        FOREIGN KEY (userid) REFERENCES users (id) ON DELETE CASCADE \
    )",
            &[],
        )
        .await
        .unwrap();

    client
        .query(
            "CREATE TABLE IF NOT EXISTS viewtokens ( \
        id text PRIMARY KEY, \
        userid text, \
        fileid text, \
        token text, \
        created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
        CONSTRAINT fk_id \
            FOREIGN KEY (userid) \
                REFERENCES users(id) ON DELETE CASCADE, \
        CONSTRAINT fk_fileid \
            FOREIGN KEY (fileid) \
                REFERENCES files(id) ON DELETE CASCADE \
    )",
            &[],
        )
        .await
        .unwrap();

    client
        .query(
            "CREATE TABLE IF NOT EXISTS events ( \
        id text PRIMARY KEY, \
        event_type text, \
        event_data text, \
        message text, \
        created_at TIMESTAMP NOT NULL DEFAULT NOW() \
    )",
            &[],
        )
        .await
        .unwrap();

    client
        .query(
            "CREATE TABLE IF NOT EXISTS upload_tokens ( \
        id text PRIMARY KEY, \
        userid text, \
        name text, \
        token text, \
        created_at TIMESTAMP NOT NULL DEFAULT NOW(), \
        description text, \
        CONSTRAINT fk_id \
            FOREIGN KEY (userid) \
                REFERENCES users(id) ON DELETE CASCADE \
    )",
            &[],
        )
        .await
        .unwrap();
}
