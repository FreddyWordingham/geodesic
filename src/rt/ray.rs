//! Ray structure.

use nalgebra::{Point3, RealField, Unit, Vector3};

/// Geometric ray structure defined by an origin point and a direction vector.
#[derive(Debug, Clone)]
pub struct Ray<T: RealField + Copy> {
    /// Starting location.
    pub origin: Point3<T>,
    /// Direction.
    pub direction: Unit<Vector3<T>>,
    /// Reciprocal of each direction component (for fast `Aabb` tests).
    pub inv_direction: Vector3<T>,
    /// Sign of each direction component (0 if ≥0 else 1), for box‐slab ordering.
    pub sign: [usize; 3],
}

impl<T: RealField + Copy> Ray<T> {
    /// Construct a new `Ray` instance.
    pub fn new(origin: Point3<T>, direction: Unit<Vector3<T>>) -> Self {
        let inv_direction = Vector3::new(T::one() / direction.x, T::one() / direction.y, T::one() / direction.z);
        let sign = [
            usize::from(inv_direction.x < T::zero()),
            usize::from(inv_direction.y < T::zero()),
            usize::from(inv_direction.z < T::zero()),
        ];

        Self {
            origin,
            direction,
            inv_direction,
            sign,
        }
    }
}
