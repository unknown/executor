mod error;
mod jobs;

use std::time::Duration;

use axum::{
    extract,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use jobs::{get_job_output, Job, JobOutput, RustJob};
use nomad_rs::Nomad;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Heartbeat" }))
        .route("/execute-rust", post(execute_rust))
        .route("/execution-output/:job_name/:job_id", get(execution_output));

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
    Success { job_id: String, job_name: String },
    Error { error: String },
}

async fn execute_rust(
    extract::Json(payload): extract::Json<ExecuteRustRequest>,
) -> (StatusCode, Json<ExecuteRustResponse>) {
    let nomad = Nomad::default();
    let job = RustJob::new(&payload.code);
    let timeout_duration = Duration::from_secs(20);
    let interval_duration = Duration::from_secs(1);

    if let Err(error) = job.submit(&nomad).await {
        eprintln!("Failed to start job: {}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ExecuteRustResponse::Error {
                error: "Failed to submit job".to_string(),
            }),
        );
    }

    // TODO: these clones??
    let nomad_clone = nomad.clone();
    let job_clone = job.clone();
    tokio::spawn(async move {
        let execution_future = job_clone.poll_job_until_dead(&nomad_clone, interval_duration);
        if let Err(_) = timeout(timeout_duration, execution_future).await {
            if let Err(error) = job_clone.stop(&nomad_clone).await {
                eprintln!("Failed to stop job: {}", error);
            }
        }
    });

    (
        StatusCode::OK,
        Json(ExecuteRustResponse::Success {
            job_id: job.job_id(),
            job_name: job.job_name(),
        }),
    )
}

#[derive(Debug, Serialize)]
#[serde(tag = "status")]
enum ExecutionOutputResponse {
    Success { output: JobOutput },
    Error { error: String },
}

async fn execution_output(
    extract::Path((job_name, job_id)): extract::Path<(String, String)>,
) -> (StatusCode, Json<ExecutionOutputResponse>) {
    let nomad = Nomad::default();
    match get_job_output(&nomad, &job_id, &job_name).await {
        Ok(output) => (
            StatusCode::OK,
            Json(ExecutionOutputResponse::Success { output }),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ExecutionOutputResponse::Error {
                error: error.to_string(),
            }),
        ),
    }
}
