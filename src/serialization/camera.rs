use nalgebra::{Point3, RealField};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::{scene::Camera, serialization::SerializedProjection};

/// Serialized representation of a `Camera`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedCamera<T: RealField + Copy> {
    /// Camera projection mode
    pub projection: SerializedProjection<T>,
    /// View point.
    pub position: [T; 3],
    /// Target position.
    pub look_at: [T; 3],
    /// Resolution of the camera in pixels (width, height).
    pub resolution: [usize; 2],
}

impl<T: RealField + Copy + ToPrimitive> SerializedCamera<T> {
    /// Construct a `Camera` instance.
    pub fn build(self) -> Camera<T> {
        let position = Point3::new(self.position[0], self.position[1], self.position[2]);
        let look_at = Point3::new(self.look_at[0], self.look_at[1], self.look_at[2]);
        let projection = self.projection.build();
        Camera::new(position, look_at, projection, self.resolution)
    }
}
