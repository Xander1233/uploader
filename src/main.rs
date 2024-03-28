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
    create_upload_token, delete_upload_token, get_detailed_upload_token, get_upload_tokens,
    regenerate_upload_token,
};
use crate::routes::dashboard::api::uploads::{get_all_uploads_of_current, get_file, upload};
use crate::routes::dashboard::api::view_tokens::create_view_token;
use crate::{
    config::settings::Settings,
    database::postgres::database_pg::connect_pg,
    routes::{
        auth::{login, logout},
        dashboard::api::account::edit_account,
        views::get_file_in_html,
    },
    util::{errors::default_catch, initialize_handlebars::init_handlebars, preflight::preflight},
};

use crate::routes::billing::api::list_subscriptions;
use crate::routes::stripe::stripe::{subscribe, webhook};
use crate::routes::views::index;
use rocket::http::Header;
use rocket::{fairing::AdHoc, fs::FileServer};
use tokio_postgres::Error;

#[macro_use]
extern crate rocket;

extern crate chrono;

#[rocket::main]
async fn main() -> Result<(), Error> {
    let mut settings = Settings::new().unwrap();

    if settings.general.is_prod {
        settings.database.dbname = format!("{}{}", settings.database.dbname, "-PROD")
    }

    let settings = settings;

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

    let stripe_client = stripe::Client::new(settings.stripe.secret_key.clone());

    let _rocket = rocket::build()
        .mount("/", routes![index])
        .mount("/view", routes![get_file_in_html])
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
                get_upload_tokens,
                get_detailed_upload_token
            ],
        )
        .mount("/public", FileServer::from("assets").rank(5))
        .mount("/", FileServer::from("frontend").rank(6))
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
        .mount(
            "/api/billing",
            routes![subscribe, list_subscriptions, webhook],
        )
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
        .attach(AdHoc::try_on_ignite("Stripe Client", |rocket| async {
            Ok(rocket.manage(stripe_client))
        }))
        .attach(AdHoc::on_response(
            "Add CORS headers to response",
            |_, response| {
                Box::pin(async move {
                    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
                    response.set_header(Header::new(
                        "Access-Control-Allow-Methods",
                        "POST, GET, PATCH, OPTIONS",
                    ));
                    response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
                    response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
                })
            },
        ))
        .launch()
        .await;

    Ok(())
}
