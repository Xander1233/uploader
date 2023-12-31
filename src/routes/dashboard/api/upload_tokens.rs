use crate::middleware::user::User;
use crate::models::tokens::upload_tokens::{
    CreateUploadTokenPayload, CreateUploadTokenReturnPayload, GetUploadTokensReturnPayload,
    UploadTokens,
};
use crate::util::string_generator::{generate_id, generate_string};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use tokio_postgres::Client;

#[post("/", data = "<data>")]
pub async fn create_upload_token(
    data: Json<CreateUploadTokenPayload>,
    user: User,
    client: &State<Client>,
) -> Result<Json<CreateUploadTokenReturnPayload>, Status> {
    let name = &data.name;
    let description = &data.description;

    let duplicate_check_result = client
        .query(
            "SELECT * FROM upload_tokens WHERE name = $1 AND userid = $2",
            &[&name, &user.id],
        )
        .await;

    if duplicate_check_result.is_err() {
        return Err(Status::InternalServerError);
    }

    let duplicate_check_rows = duplicate_check_result.unwrap();

    if !duplicate_check_rows.is_empty() {
        return Err(Status::Conflict);
    }

    let token_id = generate_id();
    let token = generate_string(Some(48));

    let create_token_result = client
        .query(
            "INSERT INTO upload_tokens (id, userid, token, name, description) VALUES ($1, $2, $3, $4, $5)",
            &[&token_id, &user.id, &token, &name, &description],
        )
        .await;

    if create_token_result.is_err() {
        return Err(Status::InternalServerError);
    }

    Ok(Json(CreateUploadTokenReturnPayload { token_id, token }))
}

#[delete("/<token_id>")]
pub async fn delete_upload_token(
    token_id: String,
    user: User,
    client: &State<Client>,
) -> Result<Status, Status> {
    let result = client
        .query(
            "DELETE FROM upload_tokens WHERE id = $1 AND userid = $2",
            &[&token_id, &user.id],
        )
        .await;

    if result.is_err() {
        return Err(Status::InternalServerError);
    }

    let rows = result.unwrap();

    if rows.is_empty() {
        return Err(Status::NotFound);
    }

    Ok(Status::Ok)
}

#[post("/<token_id>/regenerate")]
pub async fn regenerate_upload_token(
    token_id: String,
    user: User,
    client: &State<Client>,
) -> Result<Json<CreateUploadTokenReturnPayload>, Status> {
    let result = client
        .query(
            "SELECT * FROM upload_tokens WHERE id = $1 AND userid = $2",
            &[&token_id, &user.id],
        )
        .await;

    if result.is_err() {
        return Err(Status::InternalServerError);
    }

    let rows = result.unwrap();

    if rows.is_empty() {
        return Err(Status::NotFound);
    }

    let token = generate_string(Some(48));

    let result = client
        .query(
            "UPDATE upload_tokens SET token = $1 WHERE id = $2 AND userid = $3",
            &[&token, &token_id, &user.id],
        )
        .await;

    if result.is_err() {
        return Err(Status::InternalServerError);
    }

    Ok(Json(CreateUploadTokenReturnPayload { token_id, token }))
}

#[get("/")]
pub async fn get_upload_tokens(
    user: User,
    client: &State<Client>,
) -> Result<Json<GetUploadTokensReturnPayload>, Status> {
    let result = client
        .query("SELECT * FROM upload_tokens WHERE userid = $1", &[&user.id])
        .await;

    if result.is_err() {
        return Err(Status::InternalServerError);
    }

    let rows = result.unwrap();

    let mut tokens = Vec::new();

    for row in rows {
        let token_id: String = row.get("id");
        let name: String = row.get("name");
        let description: Option<String> = row.get("description");

        tokens.push(UploadTokens {
            token_id,
            name,
            description,
        });
    }

    Ok(Json(GetUploadTokensReturnPayload { tokens }))
}
