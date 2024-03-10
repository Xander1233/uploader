use rocket::data::{FromData, ToByteUnit};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{data, Data, Request};

pub struct WebhookEventPayload {
    pub contents: String,
}

#[derive(Debug)]
pub enum PayloadError {
    TooLarge,
    NoColon,
    InvalidAge,
    Io(std::io::Error),
}

#[rocket::async_trait]
impl<'r> FromData<'r> for WebhookEventPayload {
    type Error = PayloadError;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        use rocket::outcome::Outcome::*;
        use PayloadError::*;

        let limit = req
            .limits()
            .get("form")
            .unwrap_or_else(|| 1_000_000.bytes());

        let contents = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Error((Status::PayloadTooLarge, TooLarge)),
            Err(e) => return Error((Status::InternalServerError, Io(e))),
        };
        Success(WebhookEventPayload { contents })
    }
}

pub struct StripeSignature<'a> {
    pub signature: &'a str,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for StripeSignature<'r> {
    type Error = &'r str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Stripe-Signature") {
            None => Outcome::Error((Status::BadRequest, "No signature provided")),
            Some(signature) => Outcome::Success(StripeSignature { signature }),
        }
    }
}
