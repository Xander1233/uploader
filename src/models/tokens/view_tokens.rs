use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateViewTokenPayload {
    pub password: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateViewTokenReturnPayload {
    pub url: String,
    pub token: String,
}
