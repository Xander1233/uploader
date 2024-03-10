use crate::util::priceid_map::{priceid_mapping, Tiers};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use std::net::IpAddr;

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub auth: Option<String>,
    pub username: String,
    pub permission_level: i32,
    pub display_name: String,
    pub email: String,
    pub total_views: i32,
    pub total_uploads: i32,
    pub total_private_uploads: i32,
    pub total_password_protected_uploads: i32,
    pub storage_used: i32,
    pub ip: Option<IpAddr>,
    pub stripe_id: Option<String>,
    pub current_tier: Option<Tiers>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = request.headers().get_one("Authorization");

        if auth_header.is_none() {
            return Outcome::Error((Status::Unauthorized, "No auth header".to_string()));
        }

        let auth_header = auth_header.unwrap();

        let auth_header = auth_header.split(' ').collect::<Vec<&str>>();

        if auth_header.len() != 2 {
            return Outcome::Error((Status::Unauthorized, "Malformed auth header".to_string()));
        }

        let auth_header = auth_header[1];

        // Get the authheader and check if there is a user with that auth
        let client = request.rocket().state::<tokio_postgres::Client>().unwrap();

        let rows = client
            .query(
                "SELECT u.* FROM sessions s LEFT JOIN users u ON u.id = s.userid WHERE s.auth = $1",
                &[&auth_header],
            )
            .await
            .unwrap();

        if rows.len() != 1 {
            return Outcome::Error((Status::Unauthorized, "User not found".to_string()));
        }

        let id: String = rows[0].get("id");
        let username: String = rows[0].get("username");
        let display_name: String = rows[0].get("display_name");
        let email: String = rows[0].get("email");
        let permission_level: i32 = rows[0].get("permission_level");
        let total_views: i32 = rows[0].get("total_views");
        let total_uploads: i32 = rows[0].get("total_uploads");
        let total_private_uploads: i32 = rows[0].get("total_private_uploads");
        let total_password_protected_uploads: i32 = rows[0].get("total_password_protected_uploads");
        let storage_used: i32 = rows[0].get("storage_used");
        let stripe_id: Option<String> = rows[0].get("stripe_id");
        let current_tier: Option<String> = rows[0].get("current_tier");

        let current_tier = priceid_mapping(current_tier);

        let ip = request.client_ip();

        Outcome::Success(User {
            id,
            auth: Some(auth_header.to_string()),
            username,
            display_name,
            email,
            permission_level,
            total_views,
            total_uploads,
            total_private_uploads,
            total_password_protected_uploads,
            storage_used,
            ip,
            stripe_id,
            current_tier,
        })
    }
}

pub async fn get_user_via_id(id: &str, client: &State<tokio_postgres::Client>) -> User {
    let rows = client
        .query("SELECT * FROM users WHERE id = $1", &[&id])
        .await
        .unwrap();

    let username: String = rows[0].get("username");
    let id: String = rows[0].get("id");
    let display_name: String = rows[0].get("display_name");
    let email: String = rows[0].get("email");
    let permission_level: i32 = rows[0].get("permission_level");
    let total_views: i32 = rows[0].get("total_views");
    let total_uploads: i32 = rows[0].get("total_uploads");
    let storage_used: i32 = rows[0].get("storage_used");

    User {
        id,
        auth: None,
        username,
        display_name,
        email,
        permission_level,
        total_views,
        total_uploads,
        total_private_uploads: 0,
        total_password_protected_uploads: 0,
        storage_used,
        ip: None,
        stripe_id: None,
        current_tier: None,
    }
}
