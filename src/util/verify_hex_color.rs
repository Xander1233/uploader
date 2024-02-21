use regex::Regex;

pub fn verify_hex_color(color: &str) -> bool {
    let re = Regex::new(r"^#[0-9a-f]{3,6}$").unwrap();
    re.is_match(color)
}
