use crate::middleware::uploader::Uploader;
use crate::middleware::user::User;
use crate::models::uploads::payloads::{
    FileUpload, FileUploadReturnPayload, GetAllReturnPayload, Upload,
};
use crate::util::format_upload_url::format_upload_url;
use crate::util::string_generator::generate_id;
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
    let id = generate_id();

    let is_private = data.private.unwrap_or(false);

    let file_path = data.file.path().unwrap();
    let mimetype = data.file.content_type().unwrap().0.to_string();

    let mut file = std::fs::File::open(file_path).unwrap();
    let mut bytes = Vec::new();

    file.read_to_end(&mut bytes).unwrap();
    if remove_file(file_path).is_err() {
        return Err(Status::InternalServerError);
    }

    client
        .query(
            "INSERT INTO files (id, userid, data) VALUES ($1, $2, $3)",
            &[&id, &uploader.id, &bytes],
        )
        .await
        .unwrap();

    client
        .query(
            "INSERT INTO metadata (id, userid, filetype, is_private) VALUES ($1, $2, $3, $4)",
            &[&id, &uploader.id, &mimetype, &is_private],
        )
        .await
        .unwrap();

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
    client: &State<tokio_postgres::Client>,
) -> Result<(ContentType, Vec<u8>), Status> {
    let rows = client
        .query(
            "SELECT * FROM files f LEFT JOIN metadata md ON md.id = f.id WHERE f.id = $1",
            &[&id],
        )
        .await
        .unwrap();

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
                "SELECT * FROM viewtokens WHERE token = $1 AND fileid = $2",
                &[&vt, &id],
            )
            .await
            .unwrap();

        if rows.len() != 1 {
            return Err(Status::Unauthorized);
        }

        let fileid: String = rows[0].get("fileid");

        if fileid != id {
            return Err(Status::Unauthorized);
        }

        let userid: String = rows[0].get("userid");

        if userid != uid {
            return Err(Status::Unauthorized);
        }
    }

    if is_private && (user.is_none() || user.unwrap().id != uid) {
        return Err(Status::NotFound);
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
