use crate::middleware::ip::Ip;
use crate::middleware::uploader::Uploader;
use crate::middleware::user::User;
use crate::models::uploads::payloads::{
    FileUpload, FileUploadReturnPayload, GetAllReturnPayload, Upload,
};
use crate::util::format_upload_url::format_upload_url;
use crate::util::priceid_map::Tiers;
use crate::util::string_generator::generate_id;
use bcrypt::{hash, verify};
use chrono::{DateTime, Utc};
use rocket::data::ToByteUnit;
use rocket::form::Form;
use rocket::http::{ContentType, Status};
use rocket::serde::json::Json;
use rocket::State;
use std::fs::remove_file;
use std::io::Read;

#[post("/", data = "<data>")]
pub async fn upload(
    data: Form<FileUpload<'_>>,
    uploader: Uploader,
    client: &State<tokio_postgres::Client>,
) -> Result<Json<FileUploadReturnPayload>, Status> {
    if uploader.upload_token.max_uses.is_some()
        && uploader.upload_token.uses + 1 > uploader.upload_token.max_uses.unwrap()
    {
        // TODO delete token
        return Err(Status::Unauthorized);
    }

    let id = generate_id();

    let is_private = data.private.unwrap_or(false);
    let password = data.password.clone();

    let file_path = data.file.path().unwrap();
    let mimetype = data.file.content_type().unwrap().0.to_string();

    let mut file = std::fs::File::open(file_path).unwrap();
    let size = file.metadata().unwrap().len();

    let mut bytes = Vec::new();

    file.read_to_end(&mut bytes).unwrap();
    if remove_file(file_path).is_err() {
        return Err(Status::InternalServerError);
    }

    let password_copy = password.clone();

    let upload_type = {
        if password_copy.is_some() && !password_copy.unwrap().is_empty() {
            UploadType::PASSWORD
        } else if is_private {
            UploadType::PRIVATE
        } else {
            UploadType::NORMAL
        }
    };

    let tier_check = validate_upload_uses(&uploader.user, &size, &upload_type);

    if tier_check.is_err() {
        return Err(Status::Forbidden);
    }

    client
        .query(
            "INSERT INTO files (id, userid, data) VALUES ($1, $2, $3)",
            &[&id, &uploader.user.id, &bytes],
        )
        .await
        .unwrap();

    client
        .query(
            "INSERT INTO metadata (id, userid, filetype, is_private) VALUES ($1, $2, $3, $4)",
            &[&id, &uploader.user.id, &mimetype, &{
                matches!(upload_type, UploadType::PRIVATE)
            }],
        )
        .await
        .unwrap();

    if let Some(password) = password {
        if !password.is_empty() {
            let password = hash(password, 12).unwrap();

            client
                .query(
                    "UPDATE metadata SET password = $1 WHERE id = $2",
                    &[&password, &id],
                )
                .await
                .unwrap();
        }
    }

    let usage_id = generate_id();
    client
        .query(
            "INSERT INTO upload_token_uses (id, tokenid, fileid, userid) VALUES ($1, $2, $3, $4)",
            &[&usage_id, &uploader.upload_token.id, &id, &uploader.user.id],
        )
        .await
        .unwrap();

    client
        .query(
            "UPDATE upload_tokens SET uses = uses + 1 WHERE id = $1",
            &[&uploader.upload_token.id],
        )
        .await
        .unwrap();

    let private_upload_multiplier = {
        match upload_type {
            UploadType::PRIVATE => 1,
            _ => 0,
        }
    };

    let password_protected_upload_multiplier = {
        match upload_type {
            UploadType::PASSWORD => 1,
            _ => 0,
        }
    };

    let _ = client.query(
        "UPDATE users SET storage_used = storage_used + $1, total_uploads = total_uploads + $2, total_private_uploads = total_private_uploads + $3, total_password_protected_uploads = total_password_protected_uploads + $4 WHERE id = $5",
        &[&(size as i32), &1, &private_upload_multiplier, &password_protected_upload_multiplier, &uploader.user.id]
    ).await;

    Ok(Json(FileUploadReturnPayload {
        id: id.clone(),
        url: format_upload_url(id.as_str()),
    }))
}

#[get("/content/<id>?<vt>")]
pub async fn get_file(
    id: &str,
    vt: Option<String>,
    user: Option<User>,
    ip: Option<Ip>,
    client: &State<tokio_postgres::Client>,
) -> Result<(ContentType, Vec<u8>), Status> {
    let rows = client
        .query(
            "SELECT * FROM files f LEFT JOIN metadata md ON md.id = f.id WHERE f.id = $1",
            &[&id],
        )
        .await;

    if rows.is_err() {
        println!("{:?}", rows.err().unwrap());
        return Err(Status::NotFound);
    }

    let rows = rows.unwrap();

    let mimetype: String = rows[0].get("filetype");
    let bytes: Vec<u8> = rows[0].get("data");
    let is_private: bool = rows[0].get("is_private");
    let author_uid: String = rows[0].get("userid");
    let password: String = rows[0].get("password");

    let current_user_id = if let Some(user) = user {
        user.id
    } else {
        String::new()
    };

    println!("cur id {}; author id {}", current_user_id, author_uid);

    if current_user_id != author_uid {
        if is_private {
            return Err(Status::NotFound);
        }

        if !password.is_empty() {
            if vt.is_none() {
                return Err(Status::Unauthorized);
            }

            let vt = vt.unwrap();

            let rows = client
                .query(
                    "SELECT * FROM view_tokens WHERE token = $1 AND fileid = $2",
                    &[&vt, &id],
                )
                .await;

            if rows.is_err() {
                println!("{:?}", rows.err().unwrap());
                return Err(Status::Unauthorized);
            }

            let rows = rows.unwrap();

            if rows.len() != 1 {
                return Err(Status::Unauthorized);
            }

            let fileid: String = rows[0].get("fileid");

            if fileid != id {
                return Err(Status::Unauthorized);
            }

            let ip: String = ip.unwrap().0.to_string();
            let db_ip: String = rows[0].get("ip");

            let ip_verification = verify(ip, db_ip.as_str());

            if ip_verification.is_err() || !ip_verification.unwrap() {
                return Err(Status::Unauthorized);
            }
        }

        let _ = client
            .query(
                "UPDATE users SET total_views = total_views + 1 WHERE id = $1",
                &[&author_uid],
            )
            .await;

        let _ = client
            .query(
                "UPDATE metadata SET views = views + 1 WHERE id = $1",
                &[&id],
            )
            .await;
    }

    Ok((
        ContentType::parse_flexible(mimetype.as_str()).unwrap(),
        bytes,
    ))
}

#[get("/all")]
pub async fn get_all_uploads_of_current(
    user: User,
    client: &State<tokio_postgres::Client>,
) -> Result<Json<GetAllReturnPayload>, Status> {
    let rows = client
        .query("SELECT * FROM metadata WHERE userid = $1", &[&user.id])
        .await
        .unwrap();

    let mut uploads = Vec::new();

    for row in rows {
        let id: String = row.get("id");
        let is_private: bool = row.get("is_private");
        let filetype: String = row.get("filetype");
        let created_at: DateTime<Utc> = row.get("uploaded_at");

        uploads.push(Upload {
            id: id.clone(),
            url: format_upload_url(id.as_str()),
            is_private,
            filetype,
            created_at,
        });
    }

    Ok(Json(GetAllReturnPayload { uploads }))
}

#[derive(Debug)]
enum UploadType {
    NORMAL,
    PASSWORD,
    PRIVATE,
}

fn validate_upload_uses<'a>(
    user: &crate::middleware::uploader::User,
    size: &u64,
    upload_type: &UploadType,
) -> Result<bool, &'a str> {
    println!(
        "{:?} {:?} {}",
        user.current_tier, upload_type, user.total_private_uploads
    );

    let current_tier = user.current_tier.clone();

    if current_tier.is_none() {
        return Err("No tier");
    }

    match current_tier.unwrap() {
        Tiers::Free => {
            if size > &5.megabytes().as_u64() {
                return Err("Upload too big");
            }

            if user.storage_used as u64 + size > 1.gigabytes().as_u64() {
                return Err("No storage left");
            }

            match upload_type {
                UploadType::NORMAL => {}
                UploadType::PASSWORD => {
                    return Err("Free tier supports no password protected uploads")
                }
                UploadType::PRIVATE => {
                    if user.total_private_uploads > 24 {
                        return Err("No private uploads left");
                    }
                }
            }
        }
        Tiers::BaseMonthly => return check_base(user, size, upload_type),
        Tiers::BaseYearly => return check_base(user, size, upload_type),
        Tiers::StandardMonthly => return check_standard(user, size, upload_type),
        Tiers::StandardYearly => return check_standard(user, size, upload_type),
        Tiers::PlusMonthly => return check_plus(user, size, upload_type),
        Tiers::PlusYearly => return check_plus(user, size, upload_type),
        Tiers::BusinessMonthly => return check_business(user, size, upload_type),
        Tiers::BusinessYearly => return check_business(user, size, upload_type),
    };

    Err("Failed to validate current tier")
}

fn check_base<'a>(
    user: &crate::middleware::uploader::User,
    size: &u64,
    upload_type: &UploadType,
) -> Result<bool, &'a str> {
    if size > &20.megabytes().as_u64() {
        return Err("Upload too big");
    }

    if user.storage_used as u64 + size > 5.gigabytes().as_u64() {
        return Err("No storage left");
    }

    match upload_type {
        UploadType::NORMAL => {}
        UploadType::PASSWORD => {
            if user.total_password_protected_uploads > 24 {
                return Err("No password protected uploads left");
            }
        }
        UploadType::PRIVATE => {
            if user.total_private_uploads > 49 {
                return Err("No private uploads left");
            }
        }
    }

    Ok(true)
}

fn check_standard<'a>(
    user: &crate::middleware::uploader::User,
    size: &u64,
    upload_type: &UploadType,
) -> Result<bool, &'a str> {
    if size > &50.megabytes().as_u64() {
        return Err("Upload too big");
    }

    if user.storage_used as u64 + size > 15.gigabytes().as_u64() {
        return Err("No storage left");
    }

    match upload_type {
        UploadType::NORMAL => {}
        UploadType::PASSWORD => {
            if user.total_password_protected_uploads > 49 {
                return Err("No password protected uploads left");
            }
        }
        UploadType::PRIVATE => {
            if user.total_private_uploads > 99 {
                return Err("No private uploads left");
            }
        }
    }

    Ok(true)
}

fn check_plus<'a>(
    user: &crate::middleware::uploader::User,
    size: &u64,
    upload_type: &UploadType,
) -> Result<bool, &'a str> {
    if size > &100.megabytes().as_u64() {
        return Err("Upload too big");
    }

    if user.storage_used as u64 + size > 25.gigabytes().as_u64() {
        return Err("No storage left");
    }

    match upload_type {
        UploadType::NORMAL => {}
        UploadType::PASSWORD => {
            if user.total_password_protected_uploads > 199 {
                return Err("No password protected uploads left");
            }
        }
        UploadType::PRIVATE => {
            if user.total_private_uploads > 249 {
                return Err("No private uploads left");
            }
        }
    }

    Ok(true)
}

fn check_business<'a>(
    user: &crate::middleware::uploader::User,
    size: &u64,
    upload_type: &UploadType,
) -> Result<bool, &'a str> {
    if size > &500.megabytes().as_u64() {
        return Err("Upload too big");
    }

    if user.storage_used as u64 + size > 50.gigabytes().as_u64() {
        return Err("No storage left");
    }

    match upload_type {
        UploadType::NORMAL => {}
        UploadType::PASSWORD => {
            if user.total_password_protected_uploads > 699 {
                return Err("No password protected uploads left");
            }
        }
        UploadType::PRIVATE => {
            if user.total_private_uploads > 999 {
                return Err("No private uploads left");
            }
        }
    }

    Ok(true)
}
