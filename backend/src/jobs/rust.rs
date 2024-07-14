use std::env;

use async_trait::async_trait;
use nomad_rs::{
    api::job::models::JobCreateRequest,
    models::{Job, ReschedulePolicy, RestartPolicy, Task, TaskGroup, Template},
};
use serde_json::json;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RustJob {
    uuid: String,
    code: String,
}

impl RustJob {
    pub fn new(code: &str) -> Self {
        RustJob {
            uuid: Uuid::new_v4().to_string(),
            code: code.to_owned(),
        }
    }
}

#[async_trait]
impl crate::jobs::Job for RustJob {
    fn job_id(&self) -> String {
        format!("execute-rust-{}", self.uuid)
    }

    fn job_name(&self) -> String {
        "execute-rust".to_string()
    }

    fn create_job_request(&self) -> JobCreateRequest {
        JobCreateRequest {
            job: Job {
                id: Some(self.job_id()),
                name: Some(self.job_name()),
                _type: Some("batch".to_string()),

                task_groups: Some(vec![TaskGroup {
                    name: Some(self.job_name()),

                    reschedule_policy: Some(ReschedulePolicy {
                        attempts: Some(0),
                        unlimited: Some(false),
                        ..Default::default()
                    }),

                    restart_policy: Some(RestartPolicy {
                        attempts: Some(0),
                        mode: Some("fail".to_string()),
                        ..Default::default()
                    }),

                    tasks: Some(vec![Task {
                        name: Some(self.job_name()),
                        driver: Some("docker".to_string()),

                        config: Some(
                            serde_json::from_value(json!({
                                "image": env::var("RUST_IMAGE").expect("RUST_IMAGE must be set"),
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
                            embedded_tmpl: Some(self.code.to_owned()),
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
}
