use nalgebra::{RealField, Unit, Vector3};

pub struct Hit<T: RealField + Copy> {
    /// The distance to intersection.
    pub distance: T,
    /// The geometric normal at the intersection point.
    pub geometric_normal: Unit<Vector3<T>>,
    /// The Phong shading normal at the intersection point.
    pub interpolated_normal: Unit<Vector3<T>>,
}

impl<T: RealField + Copy> Hit<T> {
    /// Construct a new `Hit` instance.
    pub fn new(distance: T, geometric_normal: Unit<Vector3<T>>, interpolated_normal: Unit<Vector3<T>>) -> Self {
        debug_assert!(distance >= T::zero(), "Distance to intersection must be non-negative");
        Self {
            distance,
            geometric_normal,
            interpolated_normal,
        }
    }
}
