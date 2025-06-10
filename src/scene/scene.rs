//! Scene structure for `Ray` tracing.

use nalgebra::RealField;
use num_traits::ToPrimitive;
use std::borrow::Cow;

use crate::{
    bvh::{Bvh, BvhConfig},
    error::{Result, SceneError},
    geometry::Aabb,
    rt::{Hit, Ray},
    scene::{SceneBuilder, SceneObject},
    traits::{Bounded, Traceable},
};

/// Scene containing multiple `Traceable` objects.
#[derive(Debug)]
pub struct Scene<'a, T: RealField + Copy + ToPrimitive> {
    /// Collection of `Traceable` objects in the scene.
    objects: Vec<SceneObject<'a, T>>,
    /// `Bvh` acceleration structure for the scene.
    bvh: Bvh<T>,
}

impl<'a, T: RealField + Copy + ToPrimitive> Scene<'a, T> {
    /// Construct a new `Scene` instance.
    pub fn new(config: &BvhConfig<T>, objects: Vec<SceneObject<'a, T>>) -> Result<Self> {
        if objects.is_empty() {
            return Err(SceneError::EmptyScene.into());
        }

        let bvh = Bvh::new(config, &objects)?;
        Ok(Self { objects, bvh })
    }

    /// Return a builder for constructing a `Scene`.
    #[must_use]
    pub fn builder() -> SceneBuilder<'a, T> {
        SceneBuilder::default()
    }
}

impl<T: RealField + Copy + ToPrimitive> Bounded<T> for Scene<'_, T> {
    fn aabb(&self) -> Result<Cow<Aabb<T>>> {
        self.bvh.aabb()
    }
}

impl<T: RealField + Copy + ToPrimitive> Traceable<T> for Scene<'_, T> {
    fn intersect(&self, ray: &Ray<T>) -> Result<Option<Hit<T>>> {
        // Use the BVH to find the closest intersection
        // The BVH returns the object index within the scene
        self.bvh.intersect(ray, &self.objects).map(|opt| opt.map(|(_, hit)| hit))
    }

    fn intersect_any(&self, ray: &Ray<T>, max_distance: T) -> Result<bool> {
        self.bvh.intersect_any(ray, &self.objects, max_distance)
    }
}
