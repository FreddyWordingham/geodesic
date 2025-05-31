use nalgebra::{Point3, RealField, Unit, Vector3};
use std::borrow::Cow;

use crate::{aabb::Aabb, bounded::Bounded, hit::Hit, ray::Ray, traceable::Traceable};

/// Three-dimensional triangle with pre-computed edges and normals for fast ray intersection.
pub struct Triangle<T: RealField + Copy> {
    /// First vertex position (vertex 0).
    vertex0: Point3<T>,
    /// Vertex normals for interpolation.
    normals: [Unit<Vector3<T>>; 3],
    /// Edge from vertex 0 to vertex 1.
    edge1: Vector3<T>,
    /// Edge from vertex 0 to vertex 2.
    edge2: Vector3<T>,
    /// Geometric normal.
    geometric_normal: Unit<Vector3<T>>,
}

impl<T: RealField + Copy> Triangle<T> {
    /// Construct a new `Triangle` instance.
    pub fn new(vertices: [Point3<T>; 3], normals: [Unit<Vector3<T>>; 3]) -> Self {
        let edge1 = &vertices[1] - &vertices[0];
        let edge2 = &vertices[2] - &vertices[0];
        let geometric_normal = Unit::new_normalize(edge1.cross(&edge2));

        Self {
            vertex0: vertices[0],
            normals,
            edge1,
            edge2,
            geometric_normal,
        }
    }
}

impl<T: RealField + Copy> Bounded<T> for Triangle<T> {
    /// Compute the `Aabb` of the `Triangle`.
    fn aabb(&self) -> Cow<Aabb<T>> {
        let min_x = self
            .vertex0
            .x
            .min(self.vertex0.x + self.edge1.x)
            .min(self.vertex0.x + self.edge2.x);
        let min_y = self
            .vertex0
            .y
            .min(self.vertex0.y + self.edge1.y)
            .min(self.vertex0.y + self.edge2.y);
        let min_z = self
            .vertex0
            .z
            .min(self.vertex0.z + self.edge1.z)
            .min(self.vertex0.z + self.edge2.z);

        let max_x = self
            .vertex0
            .x
            .max(self.vertex0.x + self.edge1.x)
            .max(self.vertex0.x + self.edge2.x);
        let max_y = self
            .vertex0
            .y
            .max(self.vertex0.y + self.edge1.y)
            .max(self.vertex0.y + self.edge2.y);
        let max_z = self
            .vertex0
            .z
            .max(self.vertex0.z + self.edge1.z)
            .max(self.vertex0.z + self.edge2.z);

        Cow::Owned(Aabb::new(Point3::new(min_x, min_y, min_z), Point3::new(max_x, max_y, max_z)))
    }
}

impl<T: RealField + Copy> Traceable<T> for Triangle<T> {
    fn intersect(&self, ray: &Ray<T>) -> Option<Hit<T>> {
        let epsilon = T::default_epsilon();

        // Use pre-computed edges
        let h = ray.direction.cross(&self.edge2);
        let a = self.edge1.dot(&h);

        // If determinant is near zero, ray lies in plane of triangle
        if a > -epsilon && a < epsilon {
            return None;
        }

        let f = T::one() / a;
        let s = &ray.origin - &self.vertex0;
        let u = f * s.dot(&h);

        // Check if intersection lies outside triangle
        if u < T::zero() || u > T::one() {
            return None;
        }

        let q = s.cross(&self.edge1);
        let v = f * ray.direction.dot(&q);

        // Check if intersection lies outside triangle
        if v < T::zero() || u + v > T::one() {
            return None;
        }

        // Calculate t, ray intersects triangle
        let t = f * self.edge2.dot(&q);

        // Ray intersection behind the origin
        if t <= epsilon {
            return None;
        }

        // Calculate interpolated normal using barycentric coordinates
        let w = T::one() - u - v; // barycentric coordinate for vertex 0
        let interpolated_normal =
            Unit::new_normalize(self.normals[0].scale(w) + self.normals[1].scale(u) + self.normals[2].scale(v));

        Some(Hit::new(t, self.geometric_normal, interpolated_normal))
    }
}
