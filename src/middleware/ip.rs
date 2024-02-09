use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use std::net::IpAddr;

pub struct Ip(pub IpAddr);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Ip {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let ip = request.client_ip();

        if ip.is_none() {
            return Outcome::Error((Status::BadRequest, String::from("No ip found")));
        }

        Outcome::Success(Ip(ip.unwrap()))
    }
}
