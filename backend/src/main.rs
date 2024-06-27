mod error;
mod jobs;

use std::time::Duration;

use axum::{
    extract,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use jobs::{Job, RustJob};
use nomad_rs::Nomad;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Heartbeat" }))
        .route("/execute-rust", post(execute_rust));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct ExecuteRustRequest {
    code: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status")]
enum ExecuteRustResponse {
    Success { output: String },
    Error { error: String },
}

async fn execute_rust(
    extract::Json(payload): extract::Json<ExecuteRustRequest>,
) -> (StatusCode, Json<ExecuteRustResponse>) {
    let nomad = Nomad::default();
    let timeout = Duration::from_secs(20);
    let interval = Duration::from_secs(1);

    let job = RustJob::new(&payload.code);
    match job.execute(&nomad, timeout, interval).await {
        Ok(output) => (
            StatusCode::OK,
            Json(ExecuteRustResponse::Success { output }),
        ),
        Err(error) => {
            eprintln!("{}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExecuteRustResponse::Error {
                    error: error.to_string(),
                }),
            )
        }
    }
}
