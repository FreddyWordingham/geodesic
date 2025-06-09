use nalgebra::RealField;
use serde::{Deserialize, Serialize};

use crate::scene::Projection;

const DEGREES_TO_RADIANS: f64 = std::f64::consts::PI / 180.0;

/// Serialized representation of a `Camera`'s `Projection`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedProjection<T: RealField + Copy> {
    /// Perspective projection with a field of view (degrees).
    Perspective(T),
    /// Orthographic projection with a specified width.
    Orthographic(T),
}

impl<T: RealField + Copy> SerializedProjection<T> {
    /// Construct an `Projection` instance.
    pub fn build(self) -> Projection<T> {
        match self {
            Self::Perspective(fov) => {
                let to_rad = T::from_f64(DEGREES_TO_RADIANS).unwrap();
                Projection::Perspective(fov * to_rad)
            }
            Self::Orthographic(width) => Projection::Orthographic(width),
        }
    }
}
