use crate::config::settings::Settings;
use tokio_postgres::Client;

pub async fn preflight(client: &Client, settings: &Settings) {
    if !settings.general.is_prod && settings.database.clear {
        client
            .query("DROP TABLE IF EXISTS users CASCADE", &[])
            .await
            .unwrap();
        client
            .query("DROP TABLE IF EXISTS sessions CASCADE", &[])
            .await
            .unwrap();
        client
            .query("DROP TABLE IF EXISTS files CASCADE", &[])
            .await
            .unwrap();
        client
            .query("DROP TABLE IF EXISTS metadata CASCADE", &[])
            .await
            .unwrap();
        client
            .query("DROP TABLE IF EXISTS view_tokens CASCADE", &[])
            .await
            .unwrap();
        client
            .query("DROP TABLE IF EXISTS events CASCADE", &[])
            .await
            .unwrap();
        client
            .query("DROP TABLE IF EXISTS upload_tokens CASCADE", &[])
            .await
            .unwrap();
        client
            .query("DROP TABLE IF EXISTS upload_token_uses CASCADE", &[])
            .await
            .unwrap();
        client
            .query("DROP TABLE IF EXISTS embed_config CASCADE", &[])
            .await
            .unwrap();
    }

    client
        .query(
            "CREATE TABLE IF NOT EXISTS users ( \
        id text NOT NULL PRIMARY KEY, \
        password text NOT NULL, \
        username text UNIQUE NOT NULL, \
        permission_level int NOT NULL DEFAULT 0, \
        display_name text NOT NULL, \
        email text NOT NULL, \
        stripe_id text DEFAULT '', \
        current_tier text DEFAULT 'price_1Ori8qEbfEExjZVcPTUzocfV', \
        total_views int NOT NULL DEFAULT 0, \
        total_uploads int NOT NULL DEFAULT 0, \
        total_private_uploads int NOT NULL DEFAULT 0, \
        total_password_protected_uploads int NOT NULL DEFAULT 0, \
        storage_used int NOT NULL DEFAULT 0, \
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() \
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
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(), \
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
        uploaded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(), \
        views int DEFAULT 0, \
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
            "CREATE TABLE IF NOT EXISTS view_tokens ( \
        id text PRIMARY KEY, \
        fileid text, \
        token text, \
        ip text, \
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(), \
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
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() \
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
        max_uses int, \
        uses int DEFAULT 0, \
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(), \
        description text, \
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
            "CREATE TABLE IF NOT EXISTS upload_token_uses ( \
        id text PRIMARY KEY, \
        tokenid text, \
        userid text, \
        fileid text, \
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(), \
        CONSTRAINT fk_id \
            FOREIGN KEY (tokenid) \
                REFERENCES upload_tokens(id) ON DELETE CASCADE, \
        CONSTRAINT fk_userid \
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
            "CREATE TABLE IF NOT EXISTS embed_config ( \
        userid text PRIMARY KEY, \
        title text DEFAULT 'SparkCloud File-CDN', \
        web_title text DEFAULT 'SparkCloud File-CDN', \
        color text DEFAULT '#4b90e7', \
        background_color text DEFAULT '#f5f5f5', \
        CONSTRAINT fk_userid \
            FOREIGN KEY (userid) \
                REFERENCES users(id) ON DELETE CASCADE \
        )",
            &[],
        )
        .await
        .unwrap();

    client
        .query(
            "CREATE TABLE IF NOT EXISTS feature_flags (\
        feature text PRIMARY KEY, \
        enabled boolean NOT NULL DEFAULT false, \
        update_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(), \
        updated_by text NOT NULL DEFAULT 'system' \
        )",
            &[],
        )
        .await
        .unwrap();
}
