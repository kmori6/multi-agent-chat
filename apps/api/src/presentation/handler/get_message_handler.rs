use crate::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

pub async fn get_message_handler(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Response {
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

    let rows = match sqlx::query(
        r#"
        SELECT
            id,
            thread_id,
            speaker,
            content,
            created_at
        FROM messages
        WHERE thread_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(thread_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };

    let response: Vec<MessageResponse> = rows
        .into_iter()
        .map(|row| MessageResponse {
            id: row.get("id"),
            thread_id: row.get("thread_id"),
            role: row.get("speaker"),
            content: row.get("content"),
            created_at: row.get("created_at"),
        })
        .collect();

    (StatusCode::OK, Json(response)).into_response()
}
