use axum::{Json, Router, http::StatusCode, routing::get};
use presentation::handler::healthcheck_handler;

mod application;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/healthcheck", get(healthcheck_handler::healthcheck));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
