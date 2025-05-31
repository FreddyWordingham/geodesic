//! Axis-aligned bounding box structure.

use nalgebra::{Matrix4, Point3, RealField};
use std::{borrow::Cow, mem::swap};

use crate::{bounded::Bounded, ray::Ray};

/// Axis-aligned bounding box.
#[derive(Debug, Clone)]
pub struct Aabb<T: RealField + Copy> {
    /// Minimum corner.
    pub mins: Point3<T>,
    /// Maximum corner.
    pub maxs: Point3<T>,
}

impl<T: RealField + Copy> Aabb<T> {
    /// Construct a new `Aabb` instance.
    pub fn new(mins: Point3<T>, maxs: Point3<T>) -> Self {
        debug_assert!(
            mins.x <= maxs.x && mins.y <= maxs.y && mins.z <= maxs.z,
            "Invalid AABB bounds"
        );
        Self { mins, maxs }
    }

    /// Create an 'empty' `Aabb` with extreme bounds.
    pub fn empty() -> Self {
        let min_value = T::min_value().unwrap();
        let max_value = T::max_value().unwrap();
        Self {
            mins: Point3::new(max_value, max_value, max_value),
            maxs: Point3::new(min_value, min_value, min_value),
        }
    }

    /// Calculate the center of the `Aabb`.
    pub fn centre(&self) -> Point3<T> {
        let two = T::from_u8(2).unwrap();
        Point3::new(
            (self.mins.x + self.maxs.x) / two,
            (self.mins.y + self.maxs.y) / two,
            (self.mins.z + self.maxs.z) / two,
        )
    }

    /// Calculate the surface area of an `Aabb`.
    pub fn surface_area(&self) -> T {
        let extent = [
            self.maxs[0] - self.mins[0],
            self.maxs[1] - self.mins[1],
            self.maxs[2] - self.mins[2],
        ];
        let two = T::from_u8(2).unwrap();
        two * ((extent[0] * extent[1]) + (extent[1] * extent[2]) + (extent[2] * extent[0]))
    }

    /// Calculate the volume of an `Aabb`.
    pub fn volume(&self) -> T {
        let extent = [
            self.maxs[0] - self.mins[0],
            self.maxs[1] - self.mins[1],
            self.maxs[2] - self.mins[2],
        ];
        extent[0] * extent[1] * extent[2]
    }

    /// Return a new `Aabb` which encapsulates this `Aabb` and another `Aabb`.
    pub fn merge(&self, other: &Aabb<T>) -> Aabb<T> {
        let new_mins = Point3::new(
            self.mins.x.min(other.mins.x),
            self.mins.y.min(other.mins.y),
            self.mins.z.min(other.mins.z),
        );
        let new_maxs = Point3::new(
            self.maxs.x.max(other.maxs.x),
            self.maxs.y.max(other.maxs.y),
            self.maxs.z.max(other.maxs.z),
        );
        Aabb::new(new_mins, new_maxs)
    }

    /// Apply a transformation to the `Aabb`.
    pub fn transform(&self, transform: &Matrix4<T>) -> Aabb<T> {
        // Get all 8 corners
        let corners = [
            Point3::new(self.mins.x, self.mins.y, self.mins.z),
            Point3::new(self.maxs.x, self.mins.y, self.mins.z),
            Point3::new(self.mins.x, self.maxs.y, self.mins.z),
            Point3::new(self.maxs.x, self.maxs.y, self.mins.z),
            Point3::new(self.mins.x, self.mins.y, self.maxs.z),
            Point3::new(self.maxs.x, self.mins.y, self.maxs.z),
            Point3::new(self.mins.x, self.maxs.y, self.maxs.z),
            Point3::new(self.maxs.x, self.maxs.y, self.maxs.z),
        ];

        // Transform all corners
        let transformed_corners: Vec<Point3<T>> = corners
            .iter()
            .map(|corner| {
                let homogeneous = transform * corner.to_homogeneous();
                Point3::from_homogeneous(homogeneous).expect("Invalid transformation")
            })
            .collect();

        // Find the new min and max
        let mut min_x = transformed_corners[0].x;
        let mut min_y = transformed_corners[0].y;
        let mut min_z = transformed_corners[0].z;
        let mut max_x = transformed_corners[0].x;
        let mut max_y = transformed_corners[0].y;
        let mut max_z = transformed_corners[0].z;

        for corner in &transformed_corners[1..] {
            min_x = min_x.min(corner.x);
            min_y = min_y.min(corner.y);
            min_z = min_z.min(corner.z);
            max_x = max_x.max(corner.x);
            max_y = max_y.max(corner.y);
            max_z = max_z.max(corner.z);
        }

        Aabb::new(Point3::new(min_x, min_y, min_z), Point3::new(max_x, max_y, max_z))
    }

    /// Test for an intersection between a `Ray` and the `Aabb`.
    pub fn intersect_distance(&self, ray: &Ray<T>) -> Option<T> {
        let mut t_min = T::zero();
        let mut t_max = T::max_value().unwrap();

        for i in 0..3 {
            let ray_origin_i = ray.origin[i];
            let ray_dir_i = ray.direction[i];
            let box_min_i = self.mins[i];
            let box_max_i = self.maxs[i];

            if ray_dir_i.abs() < T::default_epsilon() {
                // Ray is parallel to the slab
                if ray_origin_i < box_min_i || ray_origin_i > box_max_i {
                    return None;
                }
            } else {
                let inv_dir = T::one() / ray_dir_i;
                let mut t0 = (box_min_i - ray_origin_i) * inv_dir;
                let mut t1 = (box_max_i - ray_origin_i) * inv_dir;

                if t0 > t1 {
                    swap(&mut t0, &mut t1);
                }

                t_min = t_min.max(t0);
                t_max = t_max.min(t1);

                if t_min > t_max {
                    return None;
                }
            }
        }

        if t_max < T::zero() {
            return None;
        }

        Some(if t_min >= T::zero() { t_min } else { t_max })
    }
}

impl<T: RealField + Copy> Bounded<T> for Aabb<T> {
    fn aabb(&self) -> Cow<Aabb<T>> {
        Cow::Borrowed(self)
    }
}
