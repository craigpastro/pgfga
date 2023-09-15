use crate::check::MAX_DEPTH;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PgFgaError {
    #[error("unexpected error")]
    SqlError(#[from] pgrx::spi::Error),

    #[error("check max depth of {MAX_DEPTH} exceeded")]
    MaxDepth,
}
