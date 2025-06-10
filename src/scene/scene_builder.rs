//! Scene structure for `Ray` tracing.

use nalgebra::{Matrix4, Point3, RealField, Unit, Vector3};
use num_traits::ToPrimitive;

use crate::{
    bvh::BvhConfig,
    error::{Result, SceneError},
    geometry::{Mesh, Sphere, Triangle},
    scene::{Instance, Scene, SceneObject},
};

/// Builder for constructing `Scene` instances.
#[derive(Debug)]
pub struct SceneBuilder<'a, T: RealField + Copy + ToPrimitive> {
    /// List of objects in the scene.
    objects: Vec<SceneObject<'a, T>>,
    /// Configuration for the `Bvh` acceleration structure.
    bvh_config: BvhConfig<T>,
}

impl<'a, T: RealField + Copy + ToPrimitive> SceneBuilder<'a, T> {
    /// Set the `Bvh` configuration for the scene
    #[must_use]
    pub const fn with_bvh_config(mut self, config: BvhConfig<T>) -> Self {
        self.bvh_config = config;
        self
    }

    /// Add a `Sphere` object to the scene.
    #[must_use]
    pub fn add_sphere(mut self, centre: Point3<T>, radius: T) -> Result<Self> {
        let sphere = Sphere::new(centre, radius)?;
        self.objects.push(SceneObject::Sphere(sphere));
        Ok(self)
    }

    /// Add a `Triangle` object to the scene.
    #[must_use]
    pub fn add_triangle(mut self, vertex_positions: [Point3<T>; 3], normals: [Unit<Vector3<T>>; 3]) -> Self {
        let triangle = Triangle::new(vertex_positions, normals);
        self.objects.push(SceneObject::Triangle(triangle));
        self
    }

    /// Add a `Instance` object to the scene.
    #[must_use]
    pub fn add_instance(mut self, mesh: &'a Mesh<T>, transform: Matrix4<T>) -> Result<Self> {
        let instance = Instance::new(mesh, transform)?;
        self.objects.push(SceneObject::Instance(instance));
        Ok(self)
    }

    /// Build the `Scene` with the current configuration and `SceneObjects`.
    pub fn build(self) -> Result<Scene<'a, T>> {
        if self.objects.is_empty() {
            return Err(SceneError::EmptyScene.into());
        }

        Scene::new(&self.bvh_config, self.objects)
    }
}

impl<T: RealField + Copy + ToPrimitive> Default for SceneBuilder<'_, T> {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            bvh_config: BvhConfig::default(),
        }
    }
}
