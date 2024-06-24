use std::time::Duration;

use async_trait::async_trait;
use nomad_rs::{
    api::job::models::{JobCreateRequest, JobListAllocationsParams},
    Nomad,
};
use tokio::time::{sleep, timeout};

use crate::error::ExecutionError;

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
    ) -> Result<String, ExecutionError> {
        let alloc_id = submit_job(nomad, &self.job_id(), &self.create_job_request()).await?;
        println!("Job has been submitted.");

        poll_job_until_dead(&nomad, &self.job_id(), timeout, interval).await?;
        println!("Job has finished.");

        get_job_output(&nomad, &alloc_id, &self.job_name()).await
    }
}

// TODO: better way to get allocation id?
pub async fn submit_job(
    nomad: &Nomad,
    job_id: &str,
    job_create_request: &JobCreateRequest,
) -> Result<String, ExecutionError> {
    nomad
        .job_create(&job_create_request)
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
        .ok_or_else(|| ExecutionError::InvalidResponse("Missing allocation ID".to_string()))
}

// TODO: use blocking queries instead of a polling interval
// see https://developer.hashicorp.com/nomad/api-docs#blocking-queries
pub async fn poll_job_until_dead(
    nomad: &Nomad,
    job_id: &str,
    timeout_duration: Duration,
    interval: Duration,
) -> Result<(), ExecutionError> {
    timeout(timeout_duration, async {
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
    .await
    .map_err(|_| ExecutionError::TimeoutError("Job's status is not dead".to_string()))?
}

pub async fn get_job_output(
    nomad: &Nomad,
    alloc_id: &str,
    job_name: &str,
) -> Result<String, ExecutionError> {
    nomad
        .client_read_file(alloc_id, &format!("alloc/logs/{}.stdout.0", job_name))
        .await
        .map_err(|error| ExecutionError::NomadError(error))
}
