use config::{Config, ConfigError};
use rocket::serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Database {
    pub host: String,
    pub port: Option<String>,
    pub user: String,
    pub password: String,
    pub dbname: String,
    pub clear: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Email {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub from: String,
    pub reply_to: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct GeneralSettings {
    pub base_url: String,
    pub is_prod: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StripeSettings {
    pub publishable_key: String,
    pub secret_key: String,
    pub webhook_signature_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub database: Database,
    pub email: Email,
    pub general: GeneralSettings,
    pub stripe: StripeSettings,
}

impl Settings {
    pub fn instance() -> Self {
        Settings::new().unwrap()
    }

    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::File::with_name("config/conf.toml").required(true))
            .build();

        if s.is_err() {
            return Err(s.err().unwrap());
        }

        s.unwrap().try_deserialize()
    }
}
