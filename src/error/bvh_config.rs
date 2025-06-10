use thiserror::Error;

/// Specific error type for `BvhConfig` validation.
#[derive(Error, Debug)]
pub enum BvhConfigError {
    #[error("Traverse cost must be positive, got: {cost}")]
    InvalidTraverseCost { cost: String },

    #[error("Intersect cost must be positive, got: {cost}")]
    InvalidIntersectCost { cost: String },

    #[error("SAH buckets must be positive, got: {buckets}")]
    InvalidSahBuckets { buckets: usize },

    #[error("Max shapes per node must be greater than 2, got: {count}")]
    InvalidMaxShapesPerNode { count: usize },

    #[error("Max depth must be positive, got: {depth}")]
    InvalidMaxDepth { depth: usize },
}
