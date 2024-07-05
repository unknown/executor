use std::time::Duration;

use async_trait::async_trait;
use nomad_rs::{
    api::job::models::{JobCreateRequest, JobListAllocationsParams, JobStopParams},
    Nomad,
};
use serde::Serialize;
use tokio::time::{sleep, timeout};

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
        timeout: Duration,
        interval: Duration,
    ) -> Result<JobOutput, ExecutionError> {
        submit_job(nomad, &self.create_job_request()).await?;
        println!("Job has been submitted.");

        poll_job_until_dead(&nomad, &self.job_id(), timeout, interval).await?;
        println!("Job has finished.");

        get_job_output(&nomad, &self.job_id(), &self.job_name()).await
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
    timeout_duration: Duration,
    interval: Duration,
) -> Result<(), ExecutionError> {
    let result = timeout(timeout_duration, async {
        loop {
            let job = nomad
                .job_read(job_id)
                .await
                .map_err(|error| ExecutionError::NomadError(error))?;

            if job.status.as_deref() == Some("dead") {
                return Ok(());
            }

            sleep(interval).await;
        }
    })
    .await;

    if let Err(_) = result {
        nomad
            .job_stop(job_id, &JobStopParams::default())
            .await
            .map_err(|_| ExecutionError::TimeoutError("Failed to stop job".to_string()))?;

        return Err(ExecutionError::TimeoutError("Job timed out".to_string()));
    }

    result.map_err(|_| ExecutionError::TimeoutError("Job timed out".to_string()))?
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

    if allocation.client_status.as_deref() != Some("complete") {
        return Err(ExecutionError::TimeoutError("Job failed".to_string()));
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
