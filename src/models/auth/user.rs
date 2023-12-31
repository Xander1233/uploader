use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: String,
    pub auth: Option<String>,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub permission_level: i32,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PublicUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub permission_level: i32,
}
