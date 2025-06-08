//! Scene structure for `Ray` tracing.

use nalgebra::RealField;
use num_traits::ToPrimitive;
use std::borrow::Cow;

use crate::prelude::*;

/// Scene containing multiple traceable objects with BVH acceleration.
pub struct Scene<'a, T: RealField + Copy + ToPrimitive> {
    /// Collection of traceable objects in the scene.
    objects: Vec<SceneObject<'a, T>>,
    /// BVH acceleration structure for the scene.
    bvh: Bvh<T>,
}

impl<'a, T: RealField + Copy + ToPrimitive> Scene<'a, T> {
    /// Construct a new `Scene` instance.
    pub fn new(config: &BvhConfig<T>, objects: Vec<SceneObject<'a, T>>) -> Self {
        assert!(!objects.is_empty(), "Scene must contain at least one object");
        let bvh = Bvh::new(config, &objects);
        Self { objects, bvh }
    }

    /// Return a builder for constructing a `Scene`.
    pub fn builder() -> SceneBuilder<'a, T> {
        SceneBuilder::new()
    }

    /// Test for an intersection between a ray and any object in the scene.
    /// Returns the closest intersection if any, along with the object index.
    pub fn intersect(&self, ray: &Ray<T>) -> Option<(usize, Hit<T>)> {
        self.bvh.intersect(ray, &self.objects)
    }

    /// Test if a ray intersects any object in the scene (shadow ray optimization).
    /// This is faster than `intersect` when you only need to know if there's any intersection.
    pub fn intersect_any(&self, ray: &Ray<T>, max_distance: T) -> bool {
        self.bvh.intersect_any(ray, &self.objects, max_distance)
    }
}

impl<'a, T: RealField + Copy + ToPrimitive> Bounded<T> for Scene<'a, T> {
    fn aabb(&self) -> Cow<Aabb<T>> {
        self.bvh.aabb()
    }
}
