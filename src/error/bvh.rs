use thiserror::Error;

/// Specific error type for `Bvh` operations.
#[derive(Debug, Clone, Copy, Error)]
pub enum BvhError {
    #[error("Bvh must contain at least one geometry, but found none.")]
    EmptyGeometry,
    #[error("Bvh must contain at least one node, but found none.")]
    EmptyNodes,
}
