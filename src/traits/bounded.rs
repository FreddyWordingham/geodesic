//! Bounded geometry trait.

use nalgebra::RealField;
use std::borrow::Cow;

use crate::geometry::Aabb;

/// Types implementing this type can be bounded by an axis-aligned bounding box (`Aabb`).
pub trait Bounded<T: RealField + Copy> {
    /// Get the axis-aligned bounding box of the geometry.
    fn aabb(&self) -> Cow<Aabb<T>>;
}
