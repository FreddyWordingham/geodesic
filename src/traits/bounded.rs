//! Collision geometry trait.

use nalgebra::RealField;
use std::borrow::Cow;

use crate::prelude::*;

/// Types implementing this type can be checked for collisions with an axis-aligned bounding box.
pub trait Bounded<T: RealField + Copy> {
    /// Get the axis-aligned bounding box of the geometry.
    fn aabb(&self) -> Cow<Aabb<T>>;
}
