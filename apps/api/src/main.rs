use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use presentation::handler::create_message_handler::create_message_handler;
use presentation::handler::create_thread_handler::create_thread_handler;
use presentation::handler::get_message_handler::get_message_handler;
use presentation::handler::healthcheck_handler::healthcheck_handler;
use presentation::handler::responses_handler::responses_handler;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
mod application;
mod domain;
mod infrastructure;
mod presentation;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");

    let state = AppState { pool };

    let app = Router::new()
        .route("/healthcheck", get(healthcheck_handler))
        .route("/responses", post(responses_handler))
        .route("/thread", post(create_thread_handler))
        .route("/thread/{id}/messages", post(create_message_handler))
        .route("/thread/{id}/messages", get(get_message_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
