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
use presentation::handler::run_thread_handler::run_thread_handler;
use presentation::handler::subscribe_thread_handler::{ThreadEventHub, subscribe_thread_handler};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
mod application;
mod domain;
mod infrastructure;
mod presentation;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub thread_events: ThreadEventHub,
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

    let state = AppState {
        pool,
        thread_events: ThreadEventHub::new(128),
    };

    let app = Router::new()
        .route("/healthcheck", get(healthcheck_handler))
        .route("/responses", post(responses_handler))
        .route("/threads", post(create_thread_handler))
        .route("/threads/{id}/messages", post(create_message_handler))
        .route("/threads/{id}/messages", get(get_message_handler))
        .route("/threads/{id}/events", get(subscribe_thread_handler))
        .route("/threads/{id}/runs", post(run_thread_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
