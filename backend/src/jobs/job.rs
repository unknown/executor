use std::time::Duration;

use async_trait::async_trait;
use nomad_rs::{
    api::job::models::{JobCreateRequest, JobListAllocationsParams, JobStopParams},
    Nomad,
};
use serde::Serialize;
use tokio::time::interval;

use crate::error::ExecutionError;

#[derive(Debug, Serialize)]
pub struct JobOutput {
    stdout: String,
    stderr: String,
}

#[async_trait]
pub trait Job {
    fn job_id(&self) -> String;
    fn job_name(&self) -> String;
    fn create_job_request(&self) -> JobCreateRequest;

    async fn execute(
        &self,
        nomad: &Nomad,
        interval_duration: Duration,
    ) -> Result<JobOutput, ExecutionError> {
        submit_job(nomad, &self.create_job_request()).await?;
        println!("Job has been submitted.");

        poll_job_until_dead(&nomad, &self.job_id(), interval_duration).await?;
        println!("Job has finished.");

        get_job_output(&nomad, &self.job_id(), &self.job_name()).await
    }

    async fn stop(&self, nomad: &Nomad) -> Result<(), ExecutionError> {
        nomad
            .job_stop(&self.job_id(), &JobStopParams::default())
            .await
            .map_err(|error| ExecutionError::NomadError(error))?;
        Ok(())
    }
}

pub async fn submit_job(
    nomad: &Nomad,
    job_create_request: &JobCreateRequest,
) -> Result<(), ExecutionError> {
    nomad
        .job_create(&job_create_request)
        .await
        .map(|_| ())
        .map_err(|error| ExecutionError::NomadError(error))
}

// TODO: use blocking queries instead of a polling interval
// see https://developer.hashicorp.com/nomad/api-docs#blocking-queries
pub async fn poll_job_until_dead(
    nomad: &Nomad,
    job_id: &str,
    interval_duration: Duration,
) -> Result<(), ExecutionError> {
    let mut interval = interval(interval_duration);

    loop {
        interval.tick().await;

        let job = nomad
            .job_read(job_id)
            .await
            .map_err(|error| ExecutionError::NomadError(error))?;

        if job.status.as_deref() == Some("dead") {
            return Ok(());
        }
    }
}

// TODO: assumes submitting job only creates one allocation
pub async fn get_job_output(
    nomad: &Nomad,
    job_id: &str,
    job_name: &str,
) -> Result<JobOutput, ExecutionError> {
    let allocations = nomad
        .job_list_allocations(job_id, &JobListAllocationsParams::default())
        .await
        .map_err(|error| ExecutionError::NomadError(error))?;
    let allocation = allocations
        .first()
        .ok_or_else(|| ExecutionError::InvalidResponse("No allocations".to_string()))?;

    // TODO: assumes task name is same as job name
    let task_state = allocation
        .task_states
        .as_ref()
        .ok_or_else(|| ExecutionError::InvalidResponse("Missing task states".to_string()))?
        .get(job_name)
        .ok_or_else(|| {
            ExecutionError::InvalidResponse(format!("Missing task state for {}", job_id))
        })?;

    let termination_event = task_state
        .events
        .as_ref()
        .ok_or_else(|| ExecutionError::InvalidResponse("Missing events".to_string()))?
        .iter()
        .find(|event| event._type.as_deref() == Some("Terminated"))
        .ok_or_else(|| ExecutionError::InvalidResponse("Missing termination event".to_string()))?;

    if !matches!(termination_event.exit_code, Some(0) | Some(1)) {
        return Err(ExecutionError::TimeoutError(
            termination_event
                .message
                .clone()
                .unwrap_or_else(|| "Job failed".to_string()),
        ));
    }

    let alloc_id = allocation
        .to_owned()
        .id
        .ok_or_else(|| ExecutionError::InvalidResponse("Missing allocation ID".to_string()))?;

    let stdout = nomad
        .client_read_file(&alloc_id, &format!("alloc/logs/{}.stdout.0", job_name))
        .await
        .map_err(|error| ExecutionError::NomadError(error))?;
    let stderr = nomad
        .client_read_file(&alloc_id, &format!("alloc/logs/{}.stderr.0", job_name))
        .await
        .map_err(|error| ExecutionError::NomadError(error))?;

    Ok(JobOutput { stdout, stderr })
}
