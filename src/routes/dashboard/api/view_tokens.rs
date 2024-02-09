use crate::config::settings::Settings;
use crate::middleware::Ip::Ip;
use crate::models::tokens::view_tokens::{CreateViewTokenPayload, CreateViewTokenReturnPayload};
use crate::util::string_generator::{generate_id, generate_string};
use bcrypt::{hash, verify};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::yansi::Paint;
use rocket::State;
use tokio_postgres::Client;

#[post("/<id>", data = "<data>")]
pub async fn create_view_token(
    id: &str,
    data: Json<CreateViewTokenPayload>,
    ip: Ip,
    client: &State<Client>,
    settings: &State<Settings>,
) -> Result<Json<CreateViewTokenReturnPayload>, Status> {
    let password = &data.password;

    let file = client
        .query("SELECT * FROM files WHERE id = $1", &[&id])
        .await;

    if file.is_err() {
        return Err(Status::NotFound);
    }

    let file = file.unwrap();

    if file.is_empty() {
        return Err(Status::NotFound);
    }

    let file = &file[0];

    let file_password: String = file.get("password");

    if verify(password, file_password.as_str()).is_err() {
        return Err(Status::Unauthorized);
    }

    let token_id = generate_id();
    let token = generate_string(Some(64));

    let ip = ip.0;

    let ip = hash(ip.to_string(), 10).unwrap();

    let create_token_result = client
        .query(
            "INSERT INTO view_tokens (id, fileid, token, ip) VALUES ($1, $2, $3, $4)",
            &[&token_id, &id, &token, &ip],
        )
        .await;

    if create_token_result.is_err() {
        return Err(Status::InternalServerError);
    }

    Ok(Json(CreateViewTokenReturnPayload {
        url: format!("{}/{}?vt={}", &settings.general.base_url, &id, &token),
        token,
    }))
}
