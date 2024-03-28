use rocket::fs::{relative, NamedFile};
use rocket::http::Status;
use rocket::serde::{json::Json, Serialize};
use rocket::Request;
use std::path::Path;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub status: u16,
    pub message: String,
}

#[catch(default)]
pub async fn default_catch<'r>(_: Status, _request: &Request<'r>) -> Option<NamedFile> {
    NamedFile::open(&Path::new(relative!("frontend")).join("index.html"))
        .await
        .ok()
}
