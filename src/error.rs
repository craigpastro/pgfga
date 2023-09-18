use crate::check::MAX_DEPTH;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PgFgaError {
    #[error("unexpected error: {0}")]
    SpiError(#[from] pgrx::spi::Error),

    #[error("error (de)serializing schema: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("check max depth of {MAX_DEPTH} exceeded")]
    MaxDepth,

    #[error("{0}")]
    Public(String),
}
