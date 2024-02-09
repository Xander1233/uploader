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
    let description = data.description.clone().unwrap_or("".to_string());
    let max_uses = &data.max_uses;

    if name.len() > 32 {
        return Err(Status::BadRequest);
    }

    if description.len() > 256 {
        return Err(Status::BadRequest);
    }

    if max_uses.is_some() && (max_uses.unwrap() > 65536) {
        return Err(Status::BadRequest);
    }

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

    let max_uses = max_uses.unwrap_or(0);

    let create_token_result = client
        .query(
            "INSERT INTO upload_tokens (id, userid, token, name, description, max_uses) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&token_id, &user.id, &token, &name, &description, &max_uses],
        )
        .await;

    if create_token_result.is_err() {
        println!("{:?}", create_token_result);
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
        let max_uses: Option<u32> = row.get("max_uses");

        tokens.push(UploadTokens {
            token_id,
            name,
            description,
            max_uses,
        });
    }

    Ok(Json(GetUploadTokensReturnPayload { tokens }))
}
