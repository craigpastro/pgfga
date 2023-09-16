use crate::check::MAX_DEPTH;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PgFgaError {
    #[error("unexpected error: {0}")]
    SqlError(#[from] pgrx::spi::Error),

    #[error("error serializing/deserializing schema: {0}")]
    DeserError(#[from] serde_json::Error),

    #[error("check max depth of {MAX_DEPTH} exceeded")]
    MaxDepth,
}
