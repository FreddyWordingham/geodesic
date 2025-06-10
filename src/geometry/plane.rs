//! Infinite plane structure.

use nalgebra::{Point3, RealField, Unit, Vector3};
use std::borrow::Cow;

use crate::{
    error::Result,
    geometry::Aabb,
    rt::{Hit, Ray},
    traits::{Bounded, FallibleNumeric, Traceable},
};

/// Infinite plane defined by a surface location and the normal vector.
#[derive(Debug, Clone)]
pub struct Plane<T: RealField + Copy> {
    /// A point on the plane.
    pub point: Point3<T>,
    /// Normal vector of the plane.
    pub normal: Unit<Vector3<T>>,
}

impl<T: RealField + Copy> Plane<T> {
    /// Construct a new `Plane` instance.
    pub const fn new(point: Point3<T>, normal: Unit<Vector3<T>>) -> Self {
        Self { point, normal }
    }

    /// Create a `Plane` from three non-collinear points.
    pub fn from_points(p1: Point3<T>, p2: Point3<T>, p3: Point3<T>) -> Self {
        let edge1 = p2 - p1;
        let edge2 = p3 - p1;
        let normal = Unit::new_normalize(edge1.cross(&edge2));
        Self::new(p1, normal)
    }

    /// Create an XY `Plane` at the given Z coordinate.
    pub fn xy_plane(z: T) -> Self {
        Self::new(Point3::new(T::zero(), T::zero(), z), Unit::new_unchecked(Vector3::z()))
    }

    /// Create an XZ `Plane` at the given Y coordinate.
    pub fn xz_plane(y: T) -> Self {
        Self::new(Point3::new(T::zero(), y, T::zero()), Unit::new_unchecked(Vector3::y()))
    }

    /// Create a YZ `Plane` at the given X coordinate.
    pub fn yz_plane(x: T) -> Self {
        Self::new(Point3::new(x, T::zero(), T::zero()), Unit::new_unchecked(Vector3::x()))
    }
}

impl<T: RealField + Copy> Bounded<T> for Plane<T> {
    fn aabb(&self) -> Result<Cow<Aabb<T>>> {
        // Infinite planes have infinite bounding boxes so we use very large values to approximate infinity
        let large_value = T::try_from_f64(1e12)?;
        Ok(Cow::Owned(Aabb::new(
            Point3::new(-large_value, -large_value, -large_value),
            Point3::new(large_value, large_value, large_value),
        )?))
    }
}

impl<T: RealField + Copy> Traceable<T> for Plane<T> {
    fn intersect(&self, ray: &Ray<T>) -> Result<Option<Hit<T>>> {
        let epsilon = T::default_epsilon();

        // Calculate the denominator of the ray-plane intersection formula
        let denominator = ray.direction.dot(&self.normal);

        // Check if ray is parallel to the plane (denominator near zero)
        if denominator.abs() < epsilon {
            return Ok(None);
        }

        // Calculate the distance along the ray to the intersection point
        let to_point = self.point - ray.origin;
        let t = to_point.dot(&self.normal) / denominator;

        // Check if intersection is behind the ray origin
        if t < epsilon {
            return Ok(None);
        }

        // For planes, geometric normal and interpolated normal are the same
        let normal = if denominator < T::zero() {
            // Ray hitting front face
            self.normal
        } else {
            // Ray hitting back face - flip normal
            Unit::new_unchecked(-self.normal.as_ref())
        };

        Ok(Some(Hit::new(t, normal, normal)?))
    }
}
