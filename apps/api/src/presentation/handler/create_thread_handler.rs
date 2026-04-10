use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateAgentRequest {
    pub name: Option<String>,
    pub persona: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CreateThreadRequest {
    pub title: Option<String>,
    pub agent_a: Option<CreateAgentRequest>,
    pub agent_b: Option<CreateAgentRequest>,
}

#[derive(Debug, Serialize)]
pub struct CreateThreadResponse {
    pub id: Uuid,
    pub title: Option<String>,
    pub status: String,
    pub agent_a: Option<CreateAgentRequest>,
    pub agent_b: Option<CreateAgentRequest>,
    pub created_at: DateTime<Utc>,
}

pub async fn create_thread_handler(Json(payload): Json<CreateThreadRequest>) -> Response {
    let title = payload.title;
    let agent_a = payload.agent_a;
    let agent_b = payload.agent_b;
    let id = Uuid::new_v4();
    let created_at = Utc::now();
    let response = CreateThreadResponse {
        id: id,
        title: title,
        status: "idle".to_string(),
        agent_a: agent_a,
        agent_b: agent_b,
        created_at: created_at,
    };
    (StatusCode::CREATED, Json(response)).into_response()
}
