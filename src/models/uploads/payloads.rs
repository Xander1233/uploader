use chrono::{DateTime, Utc};
use rocket::form::FromForm;
use rocket::fs::TempFile;
use rocket::serde::{Deserialize, Serialize};

#[derive(FromForm)]
pub struct FileUpload<'a> {
    pub file: TempFile<'a>,
    pub private: Option<bool>,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FileUploadReturnPayload {
    pub id: String,
    pub url: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GetAllReturnPayload {
    pub uploads: Vec<Upload>,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Upload {
    pub id: String,
    pub url: String,
    pub is_private: bool,
    pub filetype: String,
    pub created_at: DateTime<Utc>,
}
