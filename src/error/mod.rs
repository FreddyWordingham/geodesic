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
///
/// This enum represents all possible errors that can occur during ray tracing operations,
/// including geometry validation, scene construction, file I/O, and mathematical computations.
#[derive(Error, Debug)]
pub enum GeodesicError {
    /// Invalid geometric primitive or operation.
    ///
    /// This error occurs when creating or operating on geometric objects with invalid parameters,
    /// such as spheres with negative radii, invalid AABB bounds, or degenerate triangles.
    ///
    /// # Examples
    /// - Creating a sphere with a negative radius
    /// - Constructing an AABB where min bounds exceed max bounds
    /// - Invalid intersection distances or geometric normals
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    /// Invalid configuration parameters.
    ///
    /// This error is raised when BVH (Bounding Volume Hierarchy) or other configuration
    /// parameters are set to invalid values that would prevent proper operation.
    ///
    /// # Examples
    /// - BVH traverse cost or intersect cost set to zero or negative values
    /// - Invalid SAH bucket counts or maximum depth settings
    /// - Camera resolution set to zero dimensions
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Invalid transformation matrix or operation.
    ///
    /// This error occurs when applying transformations that are mathematically invalid
    /// or when transformation matrices cannot be properly inverted.
    ///
    /// # Examples
    /// - Attempting to invert a non-invertible transformation matrix
    /// - Invalid transformation parameters for mesh instances
    #[error("Invalid transformation: {0}")]
    InvalidTransformation(String),

    /// Error parsing mesh or scene files.
    ///
    /// This error is raised when loading or parsing external files (such as Wavefront .obj files)
    /// that contain invalid data or formatting issues.
    ///
    /// # Examples
    /// - Malformed .obj file with missing vertex data
    /// - Invalid face indices or coordinate values
    /// - Missing required file sections or unsupported file formats
    #[error("File parsing error: {0}")]
    FileParsing(String),

    /// Standard I/O operation failure.
    ///
    /// This error wraps standard library I/O errors that occur during file operations,
    /// such as reading mesh files or saving/loading serialized scenes.
    ///
    /// # Examples
    /// - File not found when loading a mesh
    /// - Permission denied when saving scene data
    /// - Disk full when writing output files
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization or deserialization failure.
    ///
    /// This error occurs when converting scenes, assets, or other structures to/from JSON format,
    /// typically during save/load operations using the `Persistable` trait.
    ///
    /// # Examples
    /// - Invalid JSON syntax when loading a scene file
    /// - Type mismatch during deserialization
    /// - Missing required fields in serialized data
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Mathematical computation error.
    ///
    /// This error is raised when mathematical operations fail, such as type conversions
    /// between numeric types or when required mathematical bounds are unavailable.
    ///
    /// # Examples
    /// - Failed conversion from f64 to the target numeric type
    /// - Numeric overflow or underflow conditions
    /// - Missing min/max value implementations for custom numeric types
    #[error("Math error: {0}")]
    Math(String),

    /// Scene construction or validation failure.
    ///
    /// This error occurs when building scenes with invalid configurations, such as
    /// empty scenes or when required assets are missing.
    ///
    /// # Examples
    /// - Attempting to create a scene with no objects
    /// - Referencing a mesh asset that doesn't exist in the asset collection
    /// - Empty BVH construction
    #[error("Scene construction error: {0}")]
    SceneConstruction(String),

    /// Asset management operation failure.
    ///
    /// This error is raised during asset loading, management, or when dealing with
    /// duplicate or missing asset identifiers.
    ///
    /// # Examples
    /// - Attempting to add an asset with a duplicate ID
    /// - Referencing an asset that hasn't been loaded
    /// - Failed mesh loading due to invalid file content
    #[error("Asset management error: {0}")]
    AssetManagement(String),
}

/// Result type alias for the ray tracing library.
pub type Result<T> = std::result::Result<T, GeodesicError>;

impl From<BvhError> for GeodesicError {
    fn from(err: BvhError) -> Self {
        Self::InvalidGeometry(err.to_string())
    }
}

impl From<BvhConfigError> for GeodesicError {
    fn from(err: BvhConfigError) -> Self {
        Self::InvalidConfiguration(err.to_string())
    }
}

impl From<FileParsingError> for GeodesicError {
    fn from(err: FileParsingError) -> Self {
        Self::FileParsing(err.to_string())
    }
}

impl From<GeometryError> for GeodesicError {
    fn from(err: GeometryError) -> Self {
        Self::InvalidGeometry(err.to_string())
    }
}

impl From<NumericError> for GeodesicError {
    fn from(err: NumericError) -> Self {
        Self::Math(err.to_string())
    }
}

impl From<SceneError> for GeodesicError {
    fn from(err: SceneError) -> Self {
        Self::SceneConstruction(err.to_string())
    }
}

impl From<TransformationError> for GeodesicError {
    fn from(err: TransformationError) -> Self {
        Self::InvalidTransformation(err.to_string())
    }
}
