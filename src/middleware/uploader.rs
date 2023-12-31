use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

#[derive(Debug)]
pub struct Uploader {
    pub id: String,
    pub auth: Option<String>,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub permission_level: i32,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Uploader {
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
                "SELECT u.* FROM upload_tokens t LEFT JOIN users u ON u.id = t.userid WHERE t.token = $1",
                &[&auth_header],
            )
            .await
            .unwrap();

        if rows.len() != 1 {
            return Outcome::Error((
                Status::Unauthorized,
                "User with upload token not found".to_string(),
            ));
        }

        let id: String = rows[0].get("id");
        let username: String = rows[0].get("username");
        let display_name: String = rows[0].get("display_name");
        let email: String = rows[0].get("email");
        let permission_level: i32 = rows[0].get("permission_level");

        Outcome::Success(Uploader {
            id,
            auth: Some(auth_header.to_string()),
            username,
            display_name,
            email,
            permission_level,
        })
    }
}
