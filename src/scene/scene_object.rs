//! Scene object structure.

use nalgebra::RealField;
use num_traits::ToPrimitive;
use std::borrow::Cow;

use crate::prelude::*;

/// Enumeration of all `Traceable` objects that can be added to a `Scene`.
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

impl<'a, T: RealField + Copy + ToPrimitive> Bounded<T> for SceneObject<'a, T> {
    fn aabb(&self) -> Cow<Aabb<T>> {
        match self {
            SceneObject::Sphere(sphere) => sphere.aabb(),
            SceneObject::Plane(plane) => plane.aabb(),
            SceneObject::Triangle(triangle) => triangle.aabb(),
            SceneObject::Mesh(mesh) => mesh.aabb(),
            SceneObject::Instance(instance) => Cow::Borrowed(instance.world_aabb()),
        }
    }
}

impl<'a, T: RealField + Copy + ToPrimitive> Traceable<T> for SceneObject<'a, T> {
    fn intersect(&self, ray: &Ray<T>) -> Option<Hit<T>> {
        match self {
            SceneObject::Sphere(sphere) => sphere.intersect(ray),
            SceneObject::Plane(plane) => plane.intersect(ray),
            SceneObject::Triangle(triangle) => triangle.intersect(ray),
            SceneObject::Mesh(mesh) => mesh.intersect(ray).map(|(_, hit)| hit),
            SceneObject::Instance(instance) => instance.intersect(ray).map(|(_, hit)| hit),
        }
    }
}
