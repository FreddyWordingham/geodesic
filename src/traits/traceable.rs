//! Traceable trait.

use nalgebra::RealField;

use crate::rt::{Hit, Ray};

/// Trait for types which can be tested for intersection by `Ray`s.
pub trait Traceable<T: RealField + Copy> {
    /// Test for an intersection between a `Ray` and this geometry.
    fn intersect(&self, ray: &Ray<T>) -> Option<Hit<T>>;
}
