use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateAgentRequest {
    pub name: String,
    pub persona: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateThreadRequest {
    pub title: String,
    pub agent_a: CreateAgentRequest,
    pub agent_b: CreateAgentRequest,
}

#[derive(Debug, Serialize)]
pub struct CreateThreadResponse {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub agent_a: CreateAgentRequest,
    pub agent_b: CreateAgentRequest,
    pub created_at: DateTime<Utc>,
}

pub async fn create_thread_handler(Json(payload): Json<CreateThreadRequest>) -> Response {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:example@localhost:5432/thread")
        .await
        .expect("failed to connect to postgres");
    let _ = sqlx::query(
        r#"
        INSERT INTO threads (
            id,
            title,
            status,
            agent_a_name,
            agent_a_persona,
            agent_b_name,
            agent_b_persona,
            created_at,
            updated_at
        )
        VALUES ($1, $2, 'idle', $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(id)
    .bind(&payload.title)
    .bind(&payload.agent_a.name)
    .bind(&payload.agent_a.persona)
    .bind(&payload.agent_b.name)
    .bind(&payload.agent_b.persona)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await;

    let response = CreateThreadResponse {
        id: id,
        title: payload.title,
        status: "idle".to_string(),
        agent_a: payload.agent_a,
        agent_b: payload.agent_b,
        created_at: now,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}
