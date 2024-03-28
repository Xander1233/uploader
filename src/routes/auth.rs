use crate::middleware::user::User;
use crate::util::string_generator::{generate_auth_token, generate_id};

use crate::feature_flags::feature_flags::FeatureFlagController;
use crate::models::auth::payloads::{LoginCredentials, LoginReturnPayload};
use bcrypt::verify;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use tokio_postgres::Client;

#[post("/login", data = "<credentials>")]
pub async fn login(
    credentials: Json<LoginCredentials>,
    client: &State<Client>,
) -> Result<Json<LoginReturnPayload>, Status> {
    let feature_flags = FeatureFlagController::new(client).await;
    let feature = feature_flags.get_feature_flag("login");

    if feature.is_none() || !feature.unwrap().enabled {
        return Err(Status::ServiceUnavailable);
    }

    let username = &credentials.username;
    let password = &credentials.password;

    println!("{} {}", username, password);

    let result = client
        .query("SELECT * FROM users WHERE username = $1", &[&username])
        .await;

    if result.is_err() {
        return Err(Status::NotFound);
    }

    let row = result.unwrap();

    println!("{:?}", row);

    let hash: String = row[0].get("password");

    if verify(password, hash.as_str()).is_err() {
        return Err(Status::Unauthorized);
    }

    let auth = generate_auth_token();
    let session_id = generate_id();
    let uid: String = row[0].get("id");

    client
        .query(
            "INSERT INTO sessions (id, auth, userid) VALUES ($1, $2, $3)",
            &[&session_id, &auth, &uid],
        )
        .await
        .unwrap();

    Ok(Json(LoginReturnPayload { token: auth }))
}

#[get("/logout")]
pub async fn logout(user: User, client: &State<Client>) -> Result<String, Status> {
    let auth = user.auth.unwrap();

    client
        .query("DELETE FROM sessions WHERE auth = $1", &[&auth])
        .await
        .unwrap();

    Ok("Logged out".to_string())
}

#[get("/check")]
pub async fn check_auth(user: Option<User>) -> Status {
    if user.is_none() {
        Status::Unauthorized
    } else {
        Status::NoContent
    }
}
