//! Sphere structure.

use nalgebra::{Point3, RealField, Unit, Vector3};
use std::borrow::Cow;

use crate::{
    geometry::Aabb,
    rt::{Hit, Ray},
    traits::{Bounded, Traceable},
};

/// Sphere structure defined by a center point and a radius.
#[derive(Debug, Clone)]
pub struct Sphere<T: RealField + Copy> {
    /// Center of the sphere.
    pub center: Point3<T>,
    /// Radius of the sphere.
    pub radius: T,
}

impl<T: RealField + Copy> Sphere<T> {
    /// Construct a new `Sphere` instance.
    pub fn new(center: Point3<T>, radius: T) -> Self {
        debug_assert!(radius >= T::zero(), "Radius must be non-negative");
        Self { center, radius }
    }
}

impl<T: RealField + Copy> Bounded<T> for Sphere<T> {
    fn aabb(&self) -> Cow<Aabb<T>> {
        let r = Vector3::new(self.radius, self.radius, self.radius);
        Cow::Owned(Aabb::new(self.center - r, self.center + r))
    }
}

impl<T: RealField + Copy> Traceable<T> for Sphere<T> {
    fn intersect(&self, ray: &Ray<T>) -> Option<Hit<T>> {
        let epsilon = T::default_epsilon();

        // Vector from ray origin to sphere center
        let oc = ray.origin - self.center;

        // Quadratic equation coefficients: at^2 + bt + c = 0
        let a = ray.direction.dot(&ray.direction);
        let b = T::from_u8(2)? * oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius * self.radius;

        // Discriminant
        let discriminant = b * b - T::from_u8(4)? * a * c;

        // No intersection if discriminant is negative
        if discriminant < T::zero() {
            return None;
        }

        let sqrt_discriminant = discriminant.sqrt();
        let two_a = T::from_u8(2)? * a;

        // Calculate both roots
        let t1 = (-b - sqrt_discriminant) / two_a;
        let t2 = (-b + sqrt_discriminant) / two_a;

        // Choose the closest positive intersection
        let t = if t1 > epsilon {
            t1
        } else if t2 > epsilon {
            t2
        } else {
            return None; // No valid intersection
        };

        // Calculate intersection point and normal
        let intersection_point = ray.origin + ray.direction.scale(t);
        let normal_vector = (intersection_point - self.center) / self.radius;
        let normal = Unit::new_normalize(normal_vector);

        Some(Hit::new(t, normal, normal))
    }
}
