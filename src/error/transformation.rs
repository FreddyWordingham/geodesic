use thiserror::Error;

/// Specific error type for transformation operations.
#[derive(Debug, Copy, Clone, Error)]
pub enum TransformationError {
    #[error("Matrix is not invertible")]
    NonInvertibleMatrix,

    #[error("Invalid transformation matrix")]
    InvalidMatrix,
}
