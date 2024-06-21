use std::env;

use nomad_client_rs::{
    api::job::models::JobCreateRequest,
    models::{Job, RestartPolicy, Task, TaskGroup, Template},
    Config, NomadClient,
};
use serde_json::json;
use uuid::Uuid;

fn create_execute_rust_job(code: &str) -> JobCreateRequest {
    JobCreateRequest {
        job: Some(Job {
            id: Some("execute-rust".to_string()),
            name: Some("execute-rust".to_string()),
            _type: Some("batch".to_string()),

            meta: Some(
                serde_json::from_value(json!({
                    "run_uuid": Uuid::new_v4().to_string()
                }))
                .unwrap(),
            ),

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

    match client.job_create(&create_execute_rust_job(code)).await {
        Ok(response) => {
            if let Some(eval_id) = response.eval_id {
                println!("Evaluation ID: {}", eval_id);
            } else {
                eprintln!("No evaluation ID");
            }
        }
        Err(error) => eprintln!("Error creating job: {}", error),
    };
}
