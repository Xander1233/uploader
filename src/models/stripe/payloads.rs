use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateSubscriptionPayload {
    pub tier: String,
    pub price: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateSubscriptionReturnPayload {
    pub url: String,
}
