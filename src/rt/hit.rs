use nalgebra::{RealField, Unit, Vector3};

use crate::error::{GeometryError, Result};

/// Records details of a ray intersection with a geometric surface.
#[derive(Debug, Clone)]
pub struct Hit<T: RealField + Copy> {
    /// Index of the internal geometry which was hit.
    pub index: usize,
    /// The distance to intersection.
    pub distance: T,
    /// The geometric normal at the intersection point.
    pub geometric_normal: Unit<Vector3<T>>,
    /// The Phong shading normal at the intersection point.
    pub interpolated_normal: Unit<Vector3<T>>,
}

impl<T: RealField + Copy> Hit<T> {
    /// Construct a new `Hit` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the distance is negative.
    pub fn new(
        index: usize,
        distance: T,
        geometric_normal: Unit<Vector3<T>>,
        interpolated_normal: Unit<Vector3<T>>,
    ) -> Result<Self> {
        if distance < T::zero() {
            return Err(GeometryError::NegativeIntersectionDistance {
                distance: distance.to_string(),
            }
            .into());
        }

        Ok(Self {
            index,
            distance,
            geometric_normal,
            interpolated_normal,
        })
    }
}
