use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateUserReturnPayload {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct EditAccountPayload {
    pub username: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct EditEmbedConfigPayload {
    pub title: Option<String>,
    pub color: Option<String>,
    pub background_color: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ChangePasswordPayload {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProfileReturnPayload {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct LoginReturnPayload {
    pub token: String,
}
