use axum::{
    Router,
    routing::{get, post},
};
use presentation::handler::create_thread_handler;
use presentation::handler::healthcheck_handler;
use presentation::handler::responses_handler;
mod application;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/healthcheck", get(healthcheck_handler::healthcheck))
        .route("/responses", post(responses_handler::responses_handler))
        .route(
            "/thread",
            post(create_thread_handler::create_thread_handler),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

use axum::{
    Json,
    body::Body,
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing_subscriber::registry::Data;
use uuid::Uuid;
