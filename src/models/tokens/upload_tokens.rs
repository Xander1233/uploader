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
    pub max_uses: Option<u32>,
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
}
