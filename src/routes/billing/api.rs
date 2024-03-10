use crate::middleware::user::User;
use rocket::http::Status;
use rocket::State;
use tokio_postgres::Client;

#[get("/list/subscriptions")]
pub async fn list_subscriptions(
    user: User,
    client: &State<Client>,
    stripe_client: &State<stripe::Client>,
) -> Result<(), Status> {
    Ok(())
}
