//! Traceable trait.

use nalgebra::RealField;

use crate::{
    error::Result,
    rt::{Hit, Ray},
};

/// Trait for types which can be tested for intersection by `Ray`s.
pub trait Traceable<T: RealField + Copy> {
    /// Test for an intersection between a `Ray` and this geometry.
    /// Returns the closest intersection if any, with the appropriate object index.
    ///
    /// # Errors
    ///
    /// Returns an error if the intersection calculation fails due to mathematical
    /// operations or invalid geometric configurations.
    fn intersect(&self, ray: &Ray<T>) -> Result<Option<Hit<T>>>;

    /// Test if a `Ray` intersects this geometry (shadow ray optimization).
    /// Returns true if there's any intersection within `max_distance`.
    ///
    /// # Errors
    ///
    /// Returns an error if the intersection test fails due to mathematical
    /// operations or invalid geometric configurations.
    fn intersect_any(&self, ray: &Ray<T>, max_distance: T) -> Result<bool> {
        // Default implementation: just check if there's a hit within range
        (self.intersect(ray)?).map_or(Ok(false), |hit| Ok(hit.distance <= max_distance))
    }
}
