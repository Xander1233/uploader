use rocket::http::Status;
use rocket::serde::{json::Json, Serialize};
use rocket::Request;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub status: u16,
    pub message: String,
}

#[catch(default)]
pub async fn default_catch<'r>(status: Status, _request: &Request<'r>) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        status: status.code,
        message: status.to_string(),
    })
}
