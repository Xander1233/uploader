use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq)]
pub enum DatabaseName {
    Users,
    Sessions,
    Files,
    FileMetadata,
    ViewTokens,
    Events,
    UploadTokens,
    UploadTokenUses,
}

pub struct Databases {}

impl Databases {
    pub fn get() -> HashMap<DatabaseName, String> {
        let mut dict = HashMap::new();

        dict.insert(DatabaseName::Users, "users".to_string());
        dict.insert(DatabaseName::Sessions, "sessions".to_string());
        dict.insert(DatabaseName::Files, "files".to_string());
        dict.insert(DatabaseName::FileMetadata, "metadata".to_string());
        dict.insert(DatabaseName::ViewTokens, "view_tokens".to_string());
        dict.insert(DatabaseName::Events, "events".to_string());
        dict.insert(DatabaseName::UploadTokens, "upload_tokens".to_string());
        dict.insert(
            DatabaseName::UploadTokenUses,
            "upload_token_uses".to_string(),
        );

        dict
    }

    pub fn get_table(name: DatabaseName) -> String {
        Databases::get().get(&name).unwrap().clone()
    }
}
