use std::env;

use nomad_client_rs::{
    api::job::models::{JobCreateRequest, JobListAllocationsParams},
    models::{Job, RestartPolicy, Task, TaskGroup, Template},
    Config, NomadClient,
};
use serde_json::json;
use uuid::Uuid;

use thiserror::Error;

#[derive(Error, Debug)]
enum ExecutionError {
    #[error("Error creating job: {0}")]
    JobError(String),
}

fn get_job_id(job_uuid: &Uuid) -> String {
    return format!("execute-rust-{}", job_uuid.to_string());
}

fn create_execute_rust_job(job_id: &str, code: &str) -> JobCreateRequest {
    JobCreateRequest {
        job: Some(Job {
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
        }),
        ..Default::default()
    }
}

async fn submit_execute_rust_job(
    client: &NomadClient,
    code: &str,
) -> Result<String, ExecutionError> {
    let job_id = get_job_id(&Uuid::new_v4());
    client
        .job_create(&create_execute_rust_job(&job_id, code))
        .await
        .map_err(|error| ExecutionError::JobError(error.to_string()))?;

    let allocations = client
        .job_list_allocations(&job_id, &JobListAllocationsParams::default())
        .await
        .map_err(|error| ExecutionError::JobError(error.to_string()))?;

    let alloc_id = allocations
        .first()
        .map(|allocation| allocation.to_owned().id)
        .ok_or_else(|| ExecutionError::JobError("No allocation created".to_string()))?
        .ok_or_else(|| ExecutionError::JobError("Allocation has no ID".to_string()))?;

    Ok(alloc_id)
}

#[tokio::main]
async fn main() {
    let base_url = env::var("NOMAD_BASE_URL").expect("Nomad base url must be defined");
    let port = env::var("NOMAD_PORT")
        .expect("Nomad port must be defined")
        .parse::<u16>()
        .expect("Nomad port must be an integer");
    let token = env::var("NOMAD_TOKEN").expect("Nomad token must be defined");

    let client = NomadClient::new(Config {
        base_url,
        port,
        token: Some(token),
        ..Default::default()
    });

    let code = r#"
    fn main() {
        println!("Hello, from Rust code!");
    }"#;

    match submit_execute_rust_job(&client, code).await {
        Ok(alloc_id) => println!("Allocation ID: {}", alloc_id),
        Err(error) => eprintln!("Error: {}", error),
    };
}
