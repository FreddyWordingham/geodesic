//! Error handling for the `Geodesic` library.

mod bvh;
mod bvh_config;
mod file_parsing;
mod geometry;
mod numeric;
mod scene;
mod transformation;

pub use bvh::BvhError;
pub use bvh_config::BvhConfigError;
pub use file_parsing::FileParsingError;
pub use geometry::GeometryError;
pub use numeric::NumericError;
pub use scene::SceneError;
pub use transformation::TransformationError;

use std::io;
use thiserror::Error;

/// Main error type for this library.
#[derive(Error, Debug)]
pub enum GeodesicError {
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Invalid transformation: {0}")]
    InvalidTransformation(String),

    #[error("File parsing error: {0}")]
    FileParsing(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Math error: {0}")]
    Math(String),

    #[error("Scene construction error: {0}")]
    SceneConstruction(String),

    #[error("Asset management error: {0}")]
    AssetManagement(String),
}

/// Result type alias for the ray tracing library.
pub type Result<T> = std::result::Result<T, GeodesicError>;

impl From<BvhError> for GeodesicError {
    fn from(err: BvhError) -> Self {
        GeodesicError::InvalidGeometry(err.to_string())
    }
}

impl From<BvhConfigError> for GeodesicError {
    fn from(err: BvhConfigError) -> Self {
        GeodesicError::InvalidConfiguration(err.to_string())
    }
}

impl From<FileParsingError> for GeodesicError {
    fn from(err: FileParsingError) -> Self {
        GeodesicError::FileParsing(err.to_string())
    }
}

impl From<GeometryError> for GeodesicError {
    fn from(err: GeometryError) -> Self {
        GeodesicError::InvalidGeometry(err.to_string())
    }
}

impl From<NumericError> for GeodesicError {
    fn from(err: NumericError) -> Self {
        GeodesicError::Math(err.to_string())
    }
}

impl From<SceneError> for GeodesicError {
    fn from(err: SceneError) -> Self {
        GeodesicError::SceneConstruction(err.to_string())
    }
}

impl From<TransformationError> for GeodesicError {
    fn from(err: TransformationError) -> Self {
        GeodesicError::InvalidTransformation(err.to_string())
    }
}
