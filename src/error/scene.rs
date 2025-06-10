use thiserror::Error;

/// Specific error type for `Scene` construction.
#[derive(Error, Debug)]
pub enum SceneError {
    #[error("Scene must contain at least one object")]
    EmptyScene,

    #[error("BVH must contain at least one geometry")]
    EmptyBvh,

    #[error("Asset with ID '{id}' already exists")]
    DuplicateAssetId { id: String },

    #[error("Asset with ID '{id}' not found")]
    AssetNotFound { id: String },
}
