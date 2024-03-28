use crate::middleware::user::{get_user_via_id, User};
use crate::util::format_upload_url::format_upload_url;
use handlebars::Handlebars;
use rocket::fs::{relative, NamedFile};
use rocket::http::{ContentType, Status};
use rocket::State;
use serde_json::json;
use std::path::Path;

#[get("/<id>?<vt>")]
pub async fn get_file_in_html<'r>(
    id: &str,
    vt: Option<String>,
    user: Option<User>,
    client: &State<tokio_postgres::Client>,
    hbs: &State<Handlebars<'r>>,
) -> Result<(ContentType, String), Status> {
    let rows = client
        .query("SELECT * FROM metadata md WHERE md.id = $1", &[&id])
        .await;

    if rows.is_err() {
        println!("{:?}", rows.err().unwrap());
        return Err(Status::NotFound);
    }

    let rows = rows.unwrap();

    println!("{:?}", rows);

    if rows.is_empty() {
        return Err(Status::NotFound);
    }

    let is_private: bool = rows[0].get("is_private");
    let uid: String = rows[0].get("userid");

    if is_private && (user.is_none() || user.unwrap().id != uid) {
        return Err(Status::NotFound);
    }

    let author = get_user_via_id(&uid, client).await;

    let embed_config_result = client
        .query(
            "SELECT * FROM embed_config WHERE userid = $1",
            &[&author.id],
        )
        .await;

    if embed_config_result.is_err() {
        return Err(Status::InternalServerError);
    }

    let embed_config_result = embed_config_result.unwrap();

    let color: String = embed_config_result[0].get("color");
    let title: String = embed_config_result[0].get("title");
    let background_color: String = embed_config_result[0].get("background_color");

    let mut file_url = format_upload_url(id);

    if vt.is_some() {
        file_url = file_url + "?vt=" + &vt.unwrap();
    }

    let rendered = hbs
        .render(
            "file",
            &json!({
                "fileid": id,
                "username": author.username,
                "fileurl": file_url,
                "plain_fileurl": file_url,
                "color": color,
                "title": title,
                "background_color": background_color,
            }),
        )
        .unwrap();

    Ok((ContentType::HTML, rendered))
}

#[get("/<_..>", rank = 25)]
pub async fn index<'r>() -> Option<NamedFile> {
    NamedFile::open(&Path::new(relative!("frontend")).join("index.html"))
        .await
        .ok()
}
