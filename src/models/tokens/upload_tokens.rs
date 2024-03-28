use chrono::{DateTime, Utc};
use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct RegenerateTokenPayload {
    pub token_id: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct RegenerateTokenReturnPayload {
    pub token_id: String,
    pub token: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateUploadTokenPayload {
    pub name: String,
    pub description: Option<String>,
    pub max_uses: Option<i32>,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateUploadTokenReturnPayload {
    pub token_id: String,
    pub token: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GetUploadTokensReturnPayload {
    pub tokens: Vec<UploadTokens>,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UploadTokens {
    pub token_id: String,
    pub name: String,
    pub description: Option<String>,
    pub max_uses: Option<i32>,
    pub uses: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DetailedUploadToken {
    pub token_id: String,
    pub name: String,
    pub description: Option<String>,
    pub max_uses: Option<i32>,
    pub uses: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub uses_arr: Vec<UploadTokenUsage>,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UploadTokenUsage {
    pub id: String,
    pub file_id: String,
    pub token_id: String,
    pub created_at: DateTime<Utc>,
}
