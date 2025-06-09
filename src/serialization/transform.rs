use nalgebra::{Matrix4, RealField, Rotation3, Translation3};
use serde::{Deserialize, Serialize};

const DEGREES_TO_RADIANS: f64 = std::f64::consts::PI / 180.0;

/// Serialized representation of a three-dimensional transformation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedTransform<T: RealField + Copy> {
    /// Translation vector [x, y, z].
    pub translation: Option<[T; 3]>,
    /// Euler rotation around axes [x, y, z] (degrees).
    pub rotation: Option<[T; 3]>,
    /// Uniform scaling factor.
    pub scale: Option<T>,
}

impl<T: RealField + Copy> SerializedTransform<T> {
    /// Construct a `Matrix4` instance.
    pub fn build(self) -> Matrix4<T> {
        let translation = self.translation.map_or_else(Translation3::identity, |translation| {
            Translation3::new(translation[0], translation[1], translation[2])
        });

        let rotation = self.rotation.map_or_else(Rotation3::identity, |rotation| {
            // Apply rotation (assuming Euler angles in radians)
            let to_rad = T::from_f64(DEGREES_TO_RADIANS).unwrap();
            Rotation3::from_euler_angles(rotation[0] * to_rad, rotation[1] * to_rad, rotation[2] * to_rad)
        });

        let scale_matrix = self.scale.map_or_else(Matrix4::identity, |scale| Matrix4::new_scaling(scale));

        // Combine transformations: Translation * Rotation * Scale
        translation.to_homogeneous() * rotation.to_homogeneous() * scale_matrix
    }
}
