use thiserror::Error;

/// Specific error type for geometry validation.
#[derive(Error, Debug)]
pub enum GeometryError {
    #[error("Invalid radius: {radius}, must be non-negative")]
    InvalidRadius { radius: String },

    #[error("Invalid resolution: width={width}, height={height}, both must be positive")]
    InvalidResolution { width: usize, height: usize },

    #[error("Invalid AABB bounds: mins=({min_x}, {min_y}, {min_z}), maxs=({max_x}, {max_y}, {max_z})")]
    InvalidAabbBounds {
        min_x: String,
        min_y: String,
        min_z: String,
        max_x: String,
        max_y: String,
        max_z: String,
    },

    #[error("Invalid intersection distance: {distance}, must be non-negative")]
    NegativeIntersectionDistance { distance: String },

    #[error("Pixel index out of bounds: [{row}, {col}], resolution: [{res_height}, {res_width}]")]
    PixelOutOfBounds {
        row: usize,
        col: usize,
        res_height: usize,
        res_width: usize,
    },
}
