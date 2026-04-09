use crate::domain::model::message::Message;
use axum::{
    Json,
    body::Body,
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize, Deserialize)]
pub struct ResponsesRequest {
    model: String,
    instruction: Option<String>,
    input: Vec<Message>,
    stream: Option<bool>,
}

pub async fn responses(Json(payload): Json<ResponsesRequest>) -> Response {
    let client = Client::new();
    let url = "http://localhost:8000/v1/responses";

    let response = match client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };

    let status = response.status();

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned();

    if content_type.starts_with("text/event-stream") {
        let stream = response.bytes_stream();
        return (
            status,
            [
                (CONTENT_TYPE, "text/event-stream"),
                (CACHE_CONTROL, "no-cache"),
            ],
            Body::from_stream(stream),
        )
            .into_response();
    }

    match response.json::<Value>().await {
        Ok(body) => (status, Json(body)).into_response(),
        Err(error) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}
