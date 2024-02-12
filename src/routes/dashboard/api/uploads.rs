use crate::middleware::ip::Ip;
use crate::middleware::uploader::Uploader;
use crate::middleware::user::User;
use crate::models::uploads::payloads::{
    FileUpload, FileUploadReturnPayload, GetAllReturnPayload, Upload,
};
use crate::util::format_upload_url::format_upload_url;
use crate::util::string_generator::generate_id;
use bcrypt::{hash, verify};
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

    if size > 10.mebibytes().as_u64() {
        return Err(Status::BadRequest);
    }

    // Postgres doesn't support unsigned integers
    if uploader.user.storage_used + (size as i32) > uploader.user.max_storage {
        return Err(Status::BadRequest);
    }

    let mut bytes = Vec::new();

    file.read_to_end(&mut bytes).unwrap();
    if remove_file(file_path).is_err() {
        return Err(Status::InternalServerError);
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
            &[&id, &uploader.user.id, &mimetype, &is_private],
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

    let _ = client.query(
        "UPDATE users SET storage_used = storage_used + $1, total_uploads = total_uploads + 1 WHERE id = $3",
        &[&(size as i32), &1 as &i32, &uploader.user.id]
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
    let uid: String = rows[0].get("userid");
    let password: String = rows[0].get("password");

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

    let current_user_id = if user.is_none() {
        "".to_string()
    } else {
        user.unwrap().id
    };

    if is_private && current_user_id != uid {
        return Err(Status::NotFound);
    }

    if current_user_id != uid {
        let _ = client
            .query(
                "UPDATE users SET total_views = total_views + 1 WHERE id = $1",
                &[&uid],
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

        uploads.push(Upload {
            id: id.clone(),
            url: format_upload_url(id.as_str()),
            is_private,
            filetype,
        });
    }

    Ok(Json(GetAllReturnPayload { uploads }))
}
