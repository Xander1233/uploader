use crate::config::settings::Settings;
use crate::middleware::stripe_event_payload::{StripeSignature, WebhookEventPayload};
use crate::middleware::user::User;
use crate::models::stripe::payloads::CreateSubscriptionPayload;
use crate::util::priceid_map::{priceid_mapping, Tiers};
use chrono::{DateTime, Utc};
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::time::Date;
use rocket::yansi::Paint;
use rocket::{custom, State};
use std::fmt::format;
use std::str::from_boxed_utf8_unchecked;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use stripe::CheckoutSessionMode::Subscription;
use stripe::{
    CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CustomerId, Event, EventType, SubscriptionId, Timestamp, Webhook,
};
use tokio_postgres::Client;

#[post("/subscribe", data = "<metadata>")]
pub async fn subscribe(
    metadata: Json<CreateSubscriptionPayload>,
    user: User,
    stripe_client: &State<stripe::Client>,
) -> Result<Redirect, Status> {
    let price = &metadata.price;

    if user.stripe_id.is_none() {
        return Err(Status::ExpectationFailed);
    }

    let checkout_session = {
        let mut params = CreateCheckoutSession::new();
        params.success_url = Some("http://localhost:8000");
        params.cancel_url = Some("http://localhost:8000/cancel");
        params.customer = Some(user.stripe_id.unwrap().parse().unwrap());
        params.mode = Some(CheckoutSessionMode::Subscription);
        params.line_items = Some(vec![CreateCheckoutSessionLineItems {
            quantity: Some(1),
            price: Some(price.to_string()),
            ..Default::default()
        }]);
        params.expand = &["line_items", "line_items.data.price.product"];

        CheckoutSession::create(stripe_client, params)
            .await
            .unwrap()
    };

    Ok(Redirect::to(checkout_session.url.unwrap()))
}

#[post("/stripe_webhook", data = "<payload>")]
pub async fn webhook(
    stripe_signature: StripeSignature<'_>,
    payload: WebhookEventPayload,
    stripe_client: &State<stripe::Client>,
    client: &State<Client>,
    settings: &State<Settings>,
) -> Status {
    if let Ok(event) = Webhook::construct_event(
        &payload.contents,
        stripe_signature.signature,
        settings.stripe.webhook_signature_secret.as_str(),
    ) {
        println!("{:?}", event.type_);

        match event.type_ {
            EventType::CheckoutSessionCompleted => {
                if let stripe::EventObject::CheckoutSession(session) = event.data.object {
                    match checkout_session_completed(stripe_client, client, session).await {
                        Ok(_) => Status::Ok,
                        Err(_) => Status::BadRequest,
                    }
                } else {
                    Status::BadRequest
                }
            }
            EventType::CustomerSubscriptionUpdated => {
                handle_subscription_update(client, event).await
            }
            EventType::CustomerSubscriptionDeleted => {
                handle_subscription_update(client, event).await
            }
            EventType::CustomerSubscriptionPaused => {
                handle_subscription_update(client, event).await
            }
            EventType::CustomerSubscriptionCreated => {
                handle_subscription_update(client, event).await
            }
            EventType::CustomerSubscriptionResumed => {
                handle_subscription_update(client, event).await
            }
            _ => Status::Accepted,
        }
    } else {
        Status::BadRequest
    }
}

async fn handle_subscription_update<'a>(client: &Client, event: Event) -> Status {
    if let stripe::EventObject::Subscription(subscription) = event.data.object {
        match handle_customer_subscription(client, &subscription).await {
            Ok(_) => Status::Ok,
            Err(_) => Status::BadRequest,
        }
    } else {
        Status::BadRequest
    }
}

async fn checkout_session_completed<'a>(
    stripe_client: &stripe::Client,
    client: &Client,
    session: CheckoutSession,
) -> Result<(), &'a str> {
    println!("Checkout Session Completed");
    println!("{:?}", session.id);

    let sub = session.subscription;

    if let Some(sub) = sub {
        let sub_id: SubscriptionId = sub.id().parse().unwrap();

        let sub = stripe::Subscription::retrieve(stripe_client, &sub_id, &[])
            .await
            .unwrap();

        let customer_id: CustomerId = sub.customer.id().parse().unwrap();
        let customer_id = customer_id.as_str();

        let first_sub_item_price = sub.items.data.first().unwrap().price.clone().unwrap();

        let price_id = first_sub_item_price.id.as_str();

        let result = client
            .query(
                "UPDATE users SET current_tier = $1 WHERE stripe_id = $2 RETURNING id, username",
                &[&price_id, &customer_id],
            )
            .await;

        if let Err(e) = result {
            println!("Failed to save subscription to db {:?}", e);
        } else {
            let rows = result.unwrap();

            let username: String = rows[0].get("username");
            let id: String = rows[0].get("id");

            println!(
                "Account {} {} subscribed to {:?}",
                id,
                username,
                priceid_mapping(Some(price_id.to_string())).unwrap_or(Tiers::Free)
            );
        }
    } else {
        println!("n/a sub")
    }

    Ok(())
}

async fn handle_customer_subscription<'a>(
    client: &Client,
    subscription: &stripe::Subscription,
) -> Result<(), &'a str> {
    let customer_id: CustomerId = subscription.customer.id().parse().unwrap();
    let customer_id = customer_id.as_str();

    if subscription.pause_collection.is_none() && subscription.canceled_at.is_none() {
        println!("Subscribing to tier {}", customer_id);

        let price_id = subscription
            .items
            .data
            .first()
            .unwrap()
            .price
            .clone()
            .unwrap()
            .id;

        let result = client
            .query(
                "UPDATE users SET current_tier = $1 WHERE stripe_id = $2",
                &[&price_id.as_str(), &customer_id],
            )
            .await;

        if let Err(e) = result {
            println!("Failed to subscribe {:?}", e);
        } else {
            println!("Subscribed to {:?}", price_id);
        }

        return Ok(());
    }
    let new_status = {
        if subscription.pause_collection.is_some() {
            "paused"
        } else {
            "cancelled"
        }
    };

    if let Some(cancel_at) = subscription.cancel_at {
        let start = SystemTime::now();
        let now_seconds = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        if cancel_at.gt(&now_seconds) {
            let cur = UNIX_EPOCH + Duration::from_secs(cancel_at as u64);

            let datetime: DateTime<Utc> = cur.into();

            println!(
                "Subscription cancellation is due on {}",
                datetime.format("%d/%m/%Y %T")
            );
            return Ok(());
        }
    }

    println!("Subscription {}", new_status);
    println!("{:?}", subscription.id);

    let result = client
        .query(
            "UPDATE users SET current_tier = $1 WHERE stripe_id = $2",
            &[
                &Some(String::from("price_1Ori8qEbfEExjZVcPTUzocfV")),
                &customer_id,
            ],
        )
        .await;

    if let Err(e) = result {
        println!(
            "Failed to {new_status} subscription for customer {:?}",
            customer_id
        );
        println!("{:?}", e);
    } else {
        println!("{new_status} subscription for stripe_user {}", customer_id);
    }

    Ok(())
}
