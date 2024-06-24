use std::time::{Duration, Instant};

use nomad_rs::{
    api::job::models::{JobCreateRequest, JobListAllocationsParams},
    models::{Job, RestartPolicy, Task, TaskGroup, Template},
    Nomad,
};
use serde_json::json;
use thiserror::Error;
use tokio::time::sleep;
use uuid::Uuid;

#[derive(Error, Debug)]
enum ExecutionError {
    #[error(transparent)]
    NomadError(nomad_rs::NomadError),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Job timed out: {0}")]
    TimeoutError(String),
}

fn get_job_id(job_uuid: &Uuid) -> String {
    return format!("execute-rust-{}", job_uuid.to_string());
}

fn create_execute_rust_job(job_id: &str, code: &str) -> JobCreateRequest {
    JobCreateRequest {
        job: Job {
            id: Some(job_id.to_string()),
            name: Some("execute-rust".to_string()),
            _type: Some("batch".to_string()),

            task_groups: Some(vec![TaskGroup {
                name: Some("execute-rust".to_string()),

                restart_policy: Some(RestartPolicy {
                    attempts: Some(0),
                    mode: Some("fail".to_string()),
                    ..Default::default()
                }),

                tasks: Some(vec![Task {
                    name: Some("execute-rust".to_string()),
                    driver: Some("docker".to_string()),

                    config: Some(
                        serde_json::from_value(json!({
                            "image": "dmo1010/executor:rust-latest",
                            "mount": [
                                {
                                    "source": "local/main.rs",
                                    "target": "/templates/rust/src/main.rs",
                                    "type": "bind"
                                }
                            ]
                        }))
                        .unwrap(),
                    ),

                    templates: Some(vec![Template {
                        embedded_tmpl: Some(code.to_string()),
                        dest_path: Some("local/main.rs".to_string()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
            ..Default::default()
        },
        ..Default::default()
    }
}

// TODO: better way to get allocation id?
async fn submit_execute_rust_job(
    nomad: &Nomad,
    job_id: &str,
    code: &str,
) -> Result<String, ExecutionError> {
    nomad
        .job_create(&create_execute_rust_job(&job_id, code))
        .await
        .map_err(|error| ExecutionError::NomadError(error))?;

    let allocations = nomad
        .job_list_allocations(job_id, &JobListAllocationsParams::default())
        .await
        .map_err(|error| ExecutionError::NomadError(error))?;

    let allocation = allocations
        .first()
        .ok_or_else(|| ExecutionError::InvalidResponse("No allocations".to_string()))?;

    allocation
        .to_owned()
        .id
        .ok_or_else(|| ExecutionError::InvalidResponse("No allocations".to_string()))
}

// TODO: use blocking queries instead of a polling interval
// see https://developer.hashicorp.com/nomad/api-docs#blocking-queries
async fn poll_job_until_dead(
    nomad: &Nomad,
    job_id: &str,
    timeout: Duration,
    interval: Duration,
) -> Result<(), ExecutionError> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        let job = nomad
            .job_read(job_id)
            .await
            .map_err(|error| ExecutionError::NomadError(error))?;

        let status = job
            .status
            .ok_or_else(|| ExecutionError::InvalidResponse("Job missing status".to_string()))?;

        if status == "dead" {
            return Ok(());
        }

        sleep(interval).await;
    }

    Err(ExecutionError::TimeoutError(
        "Job's status is not dead".to_string(),
    ))
}

async fn get_job_output(
    nomad: &Nomad,
    alloc_id: &str,
    job_name: &str,
) -> Result<String, ExecutionError> {
    nomad
        .client_read_file(alloc_id, &format!("alloc/logs/{}.stdout.0", job_name))
        .await
        .map_err(|error| ExecutionError::NomadError(error))
}

#[tokio::main]
async fn main() {
    let nomad = Nomad::default();
    let timeout = Duration::from_secs(20);
    let interval = Duration::from_secs(1);

    let code = r#"fn main() {
    println!("Hello, from Rust code!");
}"#;

    println!("Executing:\n{}", code);

    let job_id = get_job_id(&Uuid::new_v4());
    let alloc_id = submit_execute_rust_job(&nomad, &job_id, code)
        .await
        .unwrap();

    println!("Job has been submitted.");

    poll_job_until_dead(&nomad, &job_id, timeout, interval)
        .await
        .unwrap();

    println!("Job has finished.");

    let output = get_job_output(&nomad, &alloc_id, "execute-rust")
        .await
        .unwrap();

    println!("Output:\n{}", output);
}
