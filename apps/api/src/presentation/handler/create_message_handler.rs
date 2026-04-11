use crate::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateMessageRequest {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
pub struct CreateMessageResponse {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub speaker: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_message_handler(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> Response {
    if payload.content.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "content is required" })),
        )
            .into_response();
    }

    if payload.role != "user" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "role should be 'user'" })),
        )
            .into_response();
    }

    // check threads
    let threads_exists = match sqlx::query("SELECT id FROM threads WHERE id = $1")
        .bind(thread_id)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(row) => row.is_some(),
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };

    if !threads_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "thread not found" })),
        )
            .into_response();
    }

    let id = Uuid::new_v4();
    let now = Utc::now();

    let row = match sqlx::query(
        r#"
        INSERT INTO messages (
            id,
            thread_id,
            speaker,
            content,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, thread_id, speaker, content, created_at
        "#,
    )
    .bind(id)
    .bind(thread_id)
    .bind(&payload.role)
    .bind(&payload.content)
    .bind(now)
    .bind(now)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };

    let response = CreateMessageResponse {
        id: row.get("id"),
        thread_id: row.get("thread_id"),
        speaker: row.get("speaker"),
        content: row.get("content"),
        created_at: row.get("created_at"),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}
