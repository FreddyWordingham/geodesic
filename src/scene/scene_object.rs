//! Scene object structure.

use nalgebra::RealField;
use num_traits::ToPrimitive;
use std::borrow::Cow;

use crate::{
    error::Result,
    geometry::{Aabb, Mesh, Plane, Sphere, Triangle},
    rt::{Hit, Ray},
    scene::Instance,
    traits::{Bounded, Traceable},
};

/// Enumeration of all `Traceable` objects that can be added to a `Scene`.
#[derive(Debug)]
pub enum SceneObject<'a, T: RealField + Copy> {
    /// A sphere primitive.
    Sphere(Sphere<T>),
    /// An infinite plane primitive.
    Plane(Plane<T>),
    /// A triangle primitive.
    Triangle(Triangle<T>),
    /// A triangle mesh.
    Mesh(Mesh<T>),
    /// A mesh instance with transformation.
    Instance(Instance<'a, T>),
}

impl<T: RealField + Copy + ToPrimitive> Bounded<T> for SceneObject<'_, T> {
    fn aabb(&self) -> Result<Cow<Aabb<T>>> {
        match self {
            SceneObject::Sphere(sphere) => sphere.aabb(),
            SceneObject::Plane(plane) => plane.aabb(),
            SceneObject::Triangle(triangle) => triangle.aabb(),
            SceneObject::Mesh(mesh) => mesh.aabb(),
            SceneObject::Instance(instance) => Ok(Cow::Borrowed(instance.world_aabb())),
        }
    }
}

impl<T: RealField + Copy + ToPrimitive> Traceable<T> for SceneObject<'_, T> {
    fn intersect(&self, ray: &Ray<T>) -> Result<Option<Hit<T>>> {
        match self {
            SceneObject::Sphere(sphere) => sphere.intersect(ray),
            SceneObject::Plane(plane) => plane.intersect(ray),
            SceneObject::Triangle(triangle) => triangle.intersect(ray),
            SceneObject::Mesh(mesh) => mesh.intersect(ray),
            SceneObject::Instance(instance) => instance.intersect(ray),
        }
    }

    fn intersect_any(&self, ray: &Ray<T>, max_distance: T) -> Result<bool> {
        match self {
            SceneObject::Sphere(sphere) => sphere.intersect_any(ray, max_distance),
            SceneObject::Plane(plane) => plane.intersect_any(ray, max_distance),
            SceneObject::Triangle(triangle) => triangle.intersect_any(ray, max_distance),
            SceneObject::Mesh(mesh) => mesh.intersect_any(ray, max_distance),
            SceneObject::Instance(instance) => instance.intersect_any(ray, max_distance),
        }
    }
}
