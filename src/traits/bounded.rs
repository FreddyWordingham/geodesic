//! Bounded geometry trait.

use nalgebra::RealField;
use std::borrow::Cow;

use crate::{error::Result, geometry::Aabb};

/// Types implementing this type can be bounded by an axis-aligned bounding box (`Aabb`).
pub trait Bounded<T: RealField + Copy> {
    /// Get the axis-aligned bounding box of the geometry.
    ///
    /// # Errors
    ///
    /// Returns an error if the bounding box calculation fails due to invalid
    /// geometry parameters or mathematical operations.
    fn aabb(&self) -> Result<Cow<Aabb<T>>>;
}
