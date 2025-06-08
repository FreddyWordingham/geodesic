//! Traceable trait.

use nalgebra::RealField;

use crate::prelude::*;

// Trait for types which can be intersected by `Ray`s.
pub trait Traceable<T: RealField + Copy> {
    /// Test for an intersection between a `Ray` and this geometry.
    fn intersect(&self, ray: &Ray<T>) -> Option<Hit<T>>;
}
