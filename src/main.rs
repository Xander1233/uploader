mod config;
mod database;
mod email;
mod middleware;
mod models;
mod routes;
mod util;

use crate::email::notification_manager::NotificationManager;
use crate::routes::auth::check_auth;
use crate::routes::dashboard::api::account::{
    change_password, create_account, edit_embed_config, get_profile,
};
use crate::routes::dashboard::api::upload_tokens::{
    create_upload_token, delete_upload_token, get_upload_tokens, regenerate_upload_token,
};
use crate::routes::dashboard::api::uploads::{get_all_uploads_of_current, get_file, upload};
use crate::routes::dashboard::api::view_tokens::create_view_token;
use crate::{
    config::settings::Settings,
    database::postgres::database_pg::connect_pg,
    routes::{
        auth::{login, logout},
        dashboard::api::account::edit_account,
        views::{get_file_in_html, index},
    },
    util::{errors::default_catch, initialize_handlebars::init_handlebars, preflight::preflight},
};

use rocket::{fairing::AdHoc, fs::FileServer};
use tokio_postgres::Error;

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() -> Result<(), Error> {
    let settings = Settings::new().unwrap();

    let hbs = init_handlebars().await;

    let (pg_client, connection) = connect_pg().await;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        } else {
            println!("connection successful");
        }
    });

    preflight(&pg_client, &settings).await;

    let notification_manager = NotificationManager::new(settings.clone());

    let _rocket = rocket::build()
        .register("/", catchers![default_catch])
        .mount("/", routes![index, get_file_in_html])
        .mount("/api/auth", routes![login, logout, check_auth])
        .mount(
            "/api/uploads",
            routes![upload, get_file, get_all_uploads_of_current],
        )
        .mount(
            "/api/tokens",
            routes![
                create_upload_token,
                delete_upload_token,
                regenerate_upload_token,
                get_upload_tokens
            ],
        )
        .mount("/public", FileServer::from("assets"))
        .mount(
            "/api/account",
            routes![
                create_account,
                edit_account,
                change_password,
                get_profile,
                edit_embed_config
            ],
        )
        .mount("/api/view", routes![create_view_token])
        .attach(AdHoc::try_on_ignite(
            "Settings Configuration",
            |rocket| async { Ok(rocket.manage(settings)) },
        ))
        .attach(AdHoc::try_on_ignite(
            "Database Configuration",
            |rocket| async { Ok(rocket.manage(pg_client)) },
        ))
        .attach(AdHoc::try_on_ignite(
            "Handlebars Configuration",
            |rocket| async { Ok(rocket.manage(hbs)) },
        ))
        .attach(AdHoc::try_on_ignite(
            "NotificationManager",
            |rocket| async { Ok(rocket.manage(notification_manager)) },
        ))
        .launch()
        .await;

    Ok(())
}
