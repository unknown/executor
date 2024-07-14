mod error;
mod jobs;

use std::time::Duration;

use axum::{
    extract,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use jobs::{Job, JobOutput, RustJob};
use nomad_rs::Nomad;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

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
    Success { output: JobOutput },
    Error { error: String },
}

async fn execute_rust(
    extract::Json(payload): extract::Json<ExecuteRustRequest>,
) -> (StatusCode, Json<ExecuteRustResponse>) {
    struct Guard {
        nomad: Nomad,
        job: RustJob,
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            println!("Stopping job");

            let nomad = self.nomad.clone();
            let job = self.job.clone();

            tokio::spawn(async move {
                if let Err(error) = job.stop(&nomad).await {
                    eprintln!("Failed to stop job: {}", error);
                }
            });
        }
    }

    let nomad = Nomad::default();
    let job = RustJob::new(&payload.code);

    let _guard = Guard {
        nomad: nomad.clone(),
        job: job.clone(),
    };

    let timeout_duration = Duration::from_secs(20);
    let interval_duration = Duration::from_secs(1);
    match timeout(timeout_duration, job.execute(&nomad, interval_duration)).await {
        Ok(Ok(output)) => (
            StatusCode::OK,
            Json(ExecuteRustResponse::Success { output }),
        ),
        Ok(Err(error)) => {
            eprintln!("{}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExecuteRustResponse::Error {
                    error: error.to_string(),
                }),
            )
        }
        Err(_) => {
            eprintln!("Job timed out");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExecuteRustResponse::Error {
                    error: "Job timed out".to_string(),
                }),
            )
        }
    }
}
