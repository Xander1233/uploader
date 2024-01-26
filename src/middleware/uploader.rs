use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub auth: Option<String>,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub permission_level: i32,
}

#[derive(Debug)]
pub struct UploadToken {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub max_uses: Option<u32>,
    pub uses: u32,
    pub token: String,
}

#[derive(Debug)]
pub struct Uploader {
    pub user: User,
    pub upload_token: UploadToken,
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
                "SELECT u.id userid, u.username username, u.display_name display_name, u.email email, u.permission_level permission_level, t.id token_id, t.name token_name, t.description token_description, t.max_uses token_max_uses, t.uses token_uses FROM upload_tokens t LEFT JOIN users u ON u.id = t.userid WHERE t.token = $1",
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

        let userid: String = rows[0].get("userid");
        let username: String = rows[0].get("username");
        let display_name: String = rows[0].get("display_name");
        let email: String = rows[0].get("email");
        let permission_level: i32 = rows[0].get("permission_level");

        let token_id: String = rows[0].get("token_id");
        let token_name: String = rows[0].get("token_name");
        let token_desc: Option<String> = rows[0].get("token_description");
        let token_max_uses: Option<u32> = rows[0].get("token_max_uses");
        let token_uses: u32 = rows[0].get("token_uses");

        Outcome::Success(Uploader {
            user: User {
                id: userid,
                auth: None,
                username,
                display_name,
                email,
                permission_level,
            },
            upload_token: UploadToken {
                id: token_id,
                token: auth_header.to_string(),
                name: token_name,
                description: token_desc,
                max_uses: token_max_uses,
                uses: token_uses,
            },
        })
    }
}
