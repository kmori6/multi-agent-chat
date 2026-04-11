use std::os::unix::thread;

use crate::AppState;
use crate::presentation::handler::subscribe_thread_handler::ThreadEvent;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RunThreadRequest {
    pub turn_limit: Option<u32>,
}

pub async fn run_thread_handler(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Response {
    let thread_exists = match sqlx::query("SELECT id FROM threads WHERE id = $1")
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

    if !thread_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "thread not found" })),
        )
            .into_response();
    }

    let run_id = Uuid::new_v4();
    let thread_events = state.thread_events.clone();

    tokio::spawn(async move {
        thread_events
            .publish(thread_id, ThreadEvent::run_started(thread_id, run_id, 2))
            .await;

        sleep(Duration::from_secs(1)).await;

        thread_events
            .publish(
                thread_id,
                ThreadEvent::message_created(
                    thread_id,
                    run_id,
                    Uuid::new_v4(),
                    "agent_a",
                    "こんにちは、今日はどんな話をしましょうか？",
                    Utc::now(),
                ),
            )
            .await;

        sleep(Duration::from_secs(1)).await;

        thread_events
            .publish(
                thread_id,
                ThreadEvent::message_created(
                    thread_id,
                    run_id,
                    Uuid::new_v4(),
                    "agent_b",
                    "春の散歩について話したいです。",
                    Utc::now(),
                ),
            )
            .await;

        sleep(Duration::from_secs(1)).await;

        thread_events
            .publish(thread_id, ThreadEvent::run_completed(thread_id, run_id, 2))
            .await;
    });

    (
        StatusCode::ACCEPTED,
        Json(json!(
            {"status": "started", "thread_id": thread_id, "run_id": run_id
        })),
    )
        .into_response()
}
