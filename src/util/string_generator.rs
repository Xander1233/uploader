use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn generate_id() -> String {
    generate_string(Some(15))
}

pub fn generate_auth_token() -> String {
    generate_string(None)
}

pub fn generate_string(size: Option<usize>) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size.unwrap_or(48))
        .map(char::from)
        .collect()
}
