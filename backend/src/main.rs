use nomad_rs::{
    api::job::models::JobCreateRequest,
    models::{Job, RestartPolicy, Task, TaskGroup, Template},
    Nomad,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

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

async fn submit_execute_rust_job(nomad: &Nomad, code: &str) -> Result<String, ExecutionError> {
    let job_id = get_job_id(&Uuid::new_v4());
    nomad
        .job_create(&create_execute_rust_job(&job_id, code))
        .await
        .map_err(|error| ExecutionError::JobError(error.to_string()))?;
    Ok(job_id)
}

#[tokio::main]
async fn main() {
    let nomad = Nomad::default();

    let code = r#"
    fn main() {
        println!("Hello, from Rust code!");
    }"#;

    match submit_execute_rust_job(&nomad, code).await {
        Ok(alloc_id) => println!("Job ID: {}", alloc_id),
        Err(error) => eprintln!("{}", error),
    };
}
