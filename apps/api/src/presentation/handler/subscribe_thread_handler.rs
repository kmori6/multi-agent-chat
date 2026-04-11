use crate::AppState;
use async_stream::stream;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use broadcast::{Receiver, Sender};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use std::convert::Infallible;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThreadEventKind {
    RunStarted {
        run_id: Uuid,
        turn_limit: u32,
    },
    MessageCreated {
        run_id: Uuid,
        message_id: Uuid,
        role: String,
        content: String,
        created_at: DateTime<Utc>,
    },
    RunCompleted {
        run_id: Uuid,
        generated_messages: u32,
    },
    RunFailed {
        run_id: Uuid,
        error: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ThreadEvent {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub kind: ThreadEventKind,
}

impl ThreadEvent {
    pub fn run_started(thread_id: Uuid, run_id: Uuid, turn_limit: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            thread_id,
            occurred_at: Utc::now(),
            kind: ThreadEventKind::RunStarted { run_id, turn_limit },
        }
    }

    pub fn message_created(
        thread_id: Uuid,
        run_id: Uuid,
        message_id: Uuid,
        role: impl Into<String>,
        content: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            thread_id,
            occurred_at: Utc::now(),
            kind: ThreadEventKind::MessageCreated {
                run_id,
                message_id,
                role: role.into(),
                content: content.into(),
                created_at,
            },
        }
    }

    pub fn run_completed(thread_id: Uuid, run_id: Uuid, generated_messages: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            thread_id,
            occurred_at: Utc::now(),
            kind: ThreadEventKind::RunCompleted {
                run_id,
                generated_messages,
            },
        }
    }

    pub fn run_failed(thread_id: Uuid, run_id: Uuid, error: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            thread_id,
            occurred_at: Utc::now(),
            kind: ThreadEventKind::RunFailed {
                run_id,
                error: error.into(),
            },
        }
    }

    pub fn event_name(&self) -> String {
        match self.kind {
            ThreadEventKind::RunStarted { .. } => "run.started".to_string(),
            ThreadEventKind::MessageCreated { .. } => "message.created".to_string(),
            ThreadEventKind::RunCompleted { .. } => "run.completed".to_string(),
            ThreadEventKind::RunFailed { .. } => "run.failed".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ThreadEventHub {
    capacity: usize,
    channels: Arc<RwLock<HashMap<Uuid, Sender<ThreadEvent>>>>,
}

impl ThreadEventHub {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_sender(&self, thread_id: Uuid) -> Sender<ThreadEvent> {
        if let Some(sender) = self.channels.read().await.get(&thread_id).cloned() {
            return sender.clone();
        }

        let mut channels = self.channels.write().await;

        if let Some(sender) = channels.get(&thread_id).cloned() {
            return sender.clone();
        }

        let (sender, _) = broadcast::channel(self.capacity);
        channels.insert(thread_id, sender.clone());
        sender
    }

    pub async fn subscribe(&self, thread_id: Uuid) -> Receiver<ThreadEvent> {
        let sender = self.get_or_create_sender(thread_id).await;
        sender.subscribe()
    }

    pub async fn publish(&self, thread_id: Uuid, event: ThreadEvent) {
        let sender = self.get_or_create_sender(thread_id).await;

        //
        let _ = sender.send(event);
    }

    pub async fn remove_if_unused(&self, thread_id: Uuid) {
        let mut channels = self.channels.write().await;
        let should_remove = channels
            .get(&thread_id)
            .map(|sender| sender.receiver_count() == 0)
            .unwrap_or(false);

        if should_remove {
            channels.remove(&thread_id);
        }
    }
}

pub async fn subscribe_thread_handler(
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
                axum::Json(json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };

    if !thread_exists {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({ "error": "thread not found" })),
        )
            .into_response();
    }

    let mut rx = state.thread_events.subscribe(thread_id).await;
    let hub = state.thread_events.clone();

    let stream = stream! {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let data = match serde_json::to_string(&event) {
                        Ok(data) => data,
                        Err(error) => {
                            let fallback = json!({
                                "type": "serialization_error",
                                "message": error.to_string(),
                            })
                            .to_string();

                            yield Ok::<Event, Infallible>(
                                Event::default()
                                    .event("internal.error")
                                    .data(fallback),
                            );
                            continue;
                        }
                    };

                    yield Ok::<Event, Infallible>(
                        Event::default()
                        .event(event.event_name())
                        .data(data),
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    let data = json!({
                        "type": "lagged",
                        "skipped": skipped,
                    })
                    .to_string();

                    yield Ok::<Event, Infallible>(
                        Event::default()
                            .event("stream.lagged")
                            .data(data),
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
        hub.remove_if_unused(thread_id).await;
    };

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}
