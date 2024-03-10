use crate::middleware::user::User;
use crate::models::auth::payloads::{
    ChangePasswordPayload, CreateUserReturnPayload, EditAccountPayload, EditEmbedConfigPayload,
    ProfileReturnPayload, RegisterPayload,
};
use crate::util::errors::ErrorResponse;
use crate::util::priceid_map::Tiers;
use crate::util::string_generator::generate_id;
use crate::util::verify_hex_color::verify_hex_color;
use bcrypt::{hash, verify};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;

#[get("/profile")]
pub async fn get_profile(user: User) -> Option<Json<ProfileReturnPayload>> {
    let id = user.id;
    let username = user.username;
    let display_name = user.display_name;
    let email = user.email;

    Some(Json(ProfileReturnPayload {
        id,
        username,
        display_name,
        email,
    }))
}

#[post("/new", data = "<payload>")]
pub async fn create_account(
    payload: Json<RegisterPayload>,
    client: &State<Client>,
    stripe_client: &State<stripe::Client>,
) -> Result<Json<CreateUserReturnPayload>, Status> {
    let username = &payload.username;
    let password = &payload.password;
    let email = &payload.email;
    let display_name = &payload.display_name;

    let result = client
        .query(
            "SELECT * FROM users WHERE username = $1 OR email = $2",
            &[&username, &email],
        )
        .await;

    if result.is_err() {
        return Err(Status::InternalServerError);
    }

    let rows = result.unwrap();

    if !rows.is_empty() {
        return Err(Status::Conflict);
    }

    let hash = hash(password, 12).unwrap();

    let id = generate_id();

    let result = client
        .query(
            "INSERT INTO users (id, username, password, email, display_name) VALUES ($1, $2, $3, $4, $5)",
            &[&id, &username, &hash, &email, &display_name],
        )
        .await;

    if result.is_err() {
        return Err(Status::InternalServerError);
    }

    let result = client
        .query("INSERT INTO embed_config (userid) VALUES ($1)", &[&id])
        .await;

    if result.is_err() {
        return Err(Status::InternalServerError);
    }

    let stripe_customer = stripe::Customer::create(
        stripe_client.inner(),
        stripe::CreateCustomer {
            name: Some(display_name),
            email: Some(email),
            description: Some("Customer created via API/dashboard"),
            metadata: Some(std::collections::HashMap::from([(
                String::from("uid"),
                id.clone(),
            )])),

            ..Default::default()
        },
    )
    .await
    .unwrap();

    let result = client
        .query(
            "UPDATE users SET stripe_id = $1 WHERE id = $2",
            &[&stripe_customer.id.as_str(), &id],
        )
        .await
        .unwrap();

    Ok(Json(CreateUserReturnPayload {
        id: id.clone(),
        stripe_id: stripe_customer.id.to_string(),
    }))
}

#[put("/edit", data = "<payload>")]
pub async fn edit_account(
    payload: Json<EditAccountPayload>,
    user: User,
    client: &State<Client>,
) -> Json<ErrorResponse> {
    let username = &payload.username;
    let display_name = &payload.display_name;

    if username.is_none() && display_name.is_none() {
        return Json(ErrorResponse {
            status: 400,
            message: "Bad Request".to_string(),
        });
    }

    let username = username.clone().unwrap_or(user.username);
    let display_name = display_name.clone().unwrap_or(user.display_name);

    let result = client
        .query(
            "UPDATE users SET username = $1, display_name = $2 WHERE id = $3",
            &[&username, &display_name, &user.id],
        )
        .await;

    if result.is_err() {
        return Json(ErrorResponse {
            status: 500,
            message: "Internal Server Error".to_string(),
        });
    }

    Json(ErrorResponse {
        status: 200,
        message: "OK".to_string(),
    })
}

#[put("/change-password", data = "<payload>")]
pub async fn change_password(
    payload: Json<ChangePasswordPayload>,
    user: User,
    client: &State<Client>,
) -> Json<ErrorResponse> {
    let old_password = &payload.old_password;
    let new_password = &payload.new_password;

    let result = client
        .query("SELECT password FROM users WHERE id = $1", &[&user.id])
        .await;

    if result.is_err() {
        return Json(ErrorResponse {
            status: 500,
            message: "Internal Server Error".to_string(),
        });
    }

    let row = result.unwrap();

    let password_hash: String = row[0].get("password");

    if verify(old_password, password_hash.as_str()).is_err() {
        return Json(ErrorResponse {
            status: 401,
            message: "Unauthorized".to_string(),
        });
    }

    let new_password_hash = hash(new_password, 12).unwrap();

    let result = client
        .query(
            "UPDATE users SET password = $1 WHERE id = $2",
            &[&new_password_hash, &user.id],
        )
        .await;

    if result.is_err() {
        return Json(ErrorResponse {
            status: 500,
            message: "Internal Server Error".to_string(),
        });
    }

    Json(ErrorResponse {
        status: 200,
        message: "OK".to_string(),
    })
}

#[put("/embed", data = "<payload>")]
pub async fn edit_embed_config(
    payload: Json<EditEmbedConfigPayload>,
    user: User,
    client: &State<Client>,
) -> Json<ErrorResponse> {
    if user.current_tier.is_none() {
        return Json(ErrorResponse {
            status: 403,
            message: "Forbidden".to_string(),
        });
    }

    let tier = user.current_tier.unwrap() as i32;

    if tier < 1 {
        return Json(ErrorResponse {
            status: 403,
            message: "Forbidden".to_string(),
        });
    }

    let new_title = &payload.title;
    let new_web_title = &payload.web_title;
    let new_color = &payload.color;
    let new_background_color = &payload.background_color;

    if new_color.is_none()
        && new_title.is_none()
        && new_background_color.is_none()
        && new_web_title.is_none()
    {
        return Json(ErrorResponse {
            status: 400,
            message: "Bad Request".to_string(),
        });
    }

    if new_color.is_some() {
        if !verify_hex_color(new_color.clone().unwrap().as_str()) {
            return Json(ErrorResponse {
                status: 400,
                message: "Bad Request".to_string(),
            });
        }

        let result = client
            .query(
                "UPDATE embed_config SET color = $1 WHERE userid = $2",
                &[&new_color.clone().unwrap(), &user.id],
            )
            .await;

        if result.is_err() {
            return Json(ErrorResponse {
                status: 500,
                message: "Internal Server Error".to_string(),
            });
        }
    }

    if new_background_color.is_some() {
        if !verify_hex_color(new_background_color.clone().unwrap().as_str()) {
            return Json(ErrorResponse {
                status: 400,
                message: "Bad Request".to_string(),
            });
        }

        let result = client
            .query(
                "UPDATE embed_config SET background_color = $1 WHERE userid = $2",
                &[&new_background_color.clone().unwrap(), &user.id],
            )
            .await;

        if result.is_err() {
            return Json(ErrorResponse {
                status: 500,
                message: "Internal Server Error".to_string(),
            });
        }
    }

    if new_title.is_some() && tier > 2 {
        let result = client
            .query(
                "UPDATE embed_config SET title = $1 WHERE userid = $2",
                &[&new_title.clone().unwrap(), &user.id],
            )
            .await;

        if result.is_err() {
            return Json(ErrorResponse {
                status: 500,
                message: "Internal Server Error".to_string(),
            });
        }
    }

    if new_web_title.is_some() && tier > 2 {
        let result = client
            .query(
                "UPDATE embed_config SET web_title = $1 WHERE userid = $2",
                &[&new_web_title.clone().unwrap(), &user.id],
            )
            .await;

        if result.is_err() {
            return Json(ErrorResponse {
                status: 500,
                message: "Internal Server Error".to_string(),
            });
        }
    }

    Json(ErrorResponse {
        status: 200,
        message: "OK".to_string(),
    })
}
