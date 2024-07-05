use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error(transparent)]
    NomadError(nomad_rs::NomadError),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("{0}")]
    TimeoutError(String),
}
