use crate::AppState;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
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

pub async fn create_thread_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateThreadRequest>,
) -> Response {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let row = match sqlx::query(
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
        RETURNING
            id,
            title,
            status,
            agent_a_name,
            agent_a_persona,
            agent_b_name,
            agent_b_persona,
            created_at
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
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };

    let response = CreateThreadResponse {
        id: row.get("id"),
        title: row.get("title"),
        status: row.get("status"),
        agent_a: CreateAgentRequest {
            name: row.get("agent_a_name"),
            persona: row.get("agent_a_persona"),
        },
        agent_b: CreateAgentRequest {
            name: row.get("agent_b_name"),
            persona: row.get("agent_b_persona"),
        },
        created_at: row.get("created_at"),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}
