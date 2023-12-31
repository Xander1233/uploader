use crate::config::settings::Settings;

pub fn format_upload_url(id: &str) -> String {
    format!("{}/api/v/{}", Settings::instance().general.base_url, id)
}
