//! Axis-aligned bounding box structure.

use nalgebra::{Matrix4, Point3, RealField, Unit, Vector3};
use std::borrow::Cow;

use crate::{
    rt::{Hit, Ray},
    traits::{Bounded, Traceable},
};

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
    ///
    /// # Panics
    ///
    /// In practice this method will never panic.
    #[must_use]
    pub fn empty() -> Self {
        let min_value = T::min_value().unwrap();
        let max_value = T::max_value().unwrap();
        Self {
            mins: Point3::new(max_value, max_value, max_value),
            maxs: Point3::new(min_value, min_value, min_value),
        }
    }

    /// Calculate the center of the `Aabb`.
    ///
    /// # Panics
    ///
    /// In practice this method will never panic.
    pub fn centre(&self) -> Point3<T> {
        let two = T::from_u8(2).unwrap();
        Point3::new(
            (self.mins.x + self.maxs.x) / two,
            (self.mins.y + self.maxs.y) / two,
            (self.mins.z + self.maxs.z) / two,
        )
    }

    /// Calculate the surface area of an `Aabb`.
    ///
    /// # Panics
    ///
    /// In practice this method will never panic.
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
    pub fn merge(&self, other: &Self) -> Self {
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
        Self::new(new_mins, new_maxs)
    }

    /// Apply a transformation to the `Aabb`.
    pub fn transform(&self, transform: &Matrix4<T>) -> Self {
        // Instead of collecting all corners into a Vec, compute min/max on the fly
        let first_corner = Point3::new(self.mins.x, self.mins.y, self.mins.z);
        let transformed_first = transform.transform_point(&first_corner);

        let mut min_x = transformed_first.x;
        let mut min_y = transformed_first.y;
        let mut min_z = transformed_first.z;
        let mut max_x = transformed_first.x;
        let mut max_y = transformed_first.y;
        let mut max_z = transformed_first.z;

        // Transform remaining 7 corners and update min/max incrementally
        let corners = [
            (self.maxs.x, self.mins.y, self.mins.z),
            (self.mins.x, self.maxs.y, self.mins.z),
            (self.maxs.x, self.maxs.y, self.mins.z),
            (self.mins.x, self.mins.y, self.maxs.z),
            (self.maxs.x, self.mins.y, self.maxs.z),
            (self.mins.x, self.maxs.y, self.maxs.z),
            (self.maxs.x, self.maxs.y, self.maxs.z),
        ];

        for &(x, y, z) in &corners {
            let corner = Point3::new(x, y, z);
            let transformed = transform.transform_point(&corner);

            min_x = min_x.min(transformed.x);
            min_y = min_y.min(transformed.y);
            min_z = min_z.min(transformed.z);
            max_x = max_x.max(transformed.x);
            max_y = max_y.max(transformed.y);
            max_z = max_z.max(transformed.z);
        }

        Self::new(Point3::new(min_x, min_y, min_z), Point3::new(max_x, max_y, max_z))
    }

    /// Test for any intersection.
    ///
    /// # Panics
    ///
    /// In practice this method will never panic.
    pub fn intersect_any(&self, ray: &Ray<T>) -> bool {
        let mut t_min = T::zero();
        let mut t_max = T::max_value().unwrap();

        // Use pre-computed inverse directions from Ray struct
        for i in 0..3 {
            let ray_origin_i = ray.origin[i];
            let inv_dir_i = ray.inv_direction[i];
            let box_min_i = self.mins[i];
            let box_max_i = self.maxs[i];

            // Check for parallel ray (inv_direction will be inf/-inf)
            if !inv_dir_i.is_finite() {
                if ray_origin_i < box_min_i || ray_origin_i > box_max_i {
                    return false;
                }
                continue;
            }

            // Use pre-computed inverse direction
            let t0 = (box_min_i - ray_origin_i) * inv_dir_i;
            let t1 = (box_max_i - ray_origin_i) * inv_dir_i;

            // Use ray.sign for branchless min/max
            let t_near = if ray.sign[i] == 0 { t0 } else { t1 };
            let t_far = if ray.sign[i] == 0 { t1 } else { t0 };

            t_min = t_min.max(t_near);
            t_max = t_max.min(t_far);

            // Early exit if no intersection
            if t_min > t_max {
                return false;
            }
        }

        // If the maximum distance is negative, the box is behind the ray
        match t_max.partial_cmp(&T::zero()) {
            Some(std::cmp::Ordering::Less) => false,
            Some(_) => true,
            None => unimplemented!("t_max is NaN, cannot determine intersection"),
        }
    }

    /// Test for an intersection between a `Ray` and the `Aabb`.
    ///
    /// # Panics
    ///
    /// In practice this method will never panic.
    pub fn intersect_distance(&self, ray: &Ray<T>) -> Option<T> {
        let mut t_min = T::zero();
        let mut t_max = T::max_value().unwrap();

        // Use pre-computed inverse directions from Ray struct
        for i in 0..3 {
            let ray_origin_i = ray.origin[i];
            let inv_dir_i = ray.inv_direction[i];
            let box_min_i = self.mins[i];
            let box_max_i = self.maxs[i];

            // Check for parallel ray (inv_direction will be inf/-inf)
            if !inv_dir_i.is_finite() {
                if ray_origin_i < box_min_i || ray_origin_i > box_max_i {
                    return None;
                }
                continue;
            }

            // Use pre-computed inverse direction
            let t0 = (box_min_i - ray_origin_i) * inv_dir_i;
            let t1 = (box_max_i - ray_origin_i) * inv_dir_i;

            // Use ray.sign for branchless min/max
            let t_near = if ray.sign[i] == 0 { t0 } else { t1 };
            let t_far = if ray.sign[i] == 0 { t1 } else { t0 };

            t_min = t_min.max(t_near);
            t_max = t_max.min(t_far);

            // Early exit if no intersection
            if t_min > t_max {
                return None;
            }
        }

        if t_max < T::zero() {
            return None;
        }

        Some(if t_min >= T::zero() { t_min } else { t_max })
    }
}

impl<T: RealField + Copy> Bounded<T> for Aabb<T> {
    fn aabb(&self) -> Cow<Self> {
        Cow::Borrowed(self)
    }
}

impl<T: RealField + Copy> Traceable<T> for Aabb<T> {
    fn intersect(&self, ray: &Ray<T>) -> Option<Hit<T>> {
        let mut t_min = T::zero();
        let mut t_max = T::max_value().unwrap();
        let mut normal_axis = 0;
        let mut normal_sign = T::one();

        // Use pre-computed inverse directions from Ray struct
        for i in 0..3 {
            let ray_origin_i = ray.origin[i];
            let inv_dir_i = ray.inv_direction[i];
            let box_min_i = self.mins[i];
            let box_max_i = self.maxs[i];

            // Check for parallel ray (inv_direction will be inf/-inf)
            if !inv_dir_i.is_finite() {
                if ray_origin_i < box_min_i || ray_origin_i > box_max_i {
                    return None;
                }
                continue;
            }

            // Use pre-computed inverse direction
            let t0 = (box_min_i - ray_origin_i) * inv_dir_i;
            let t1 = (box_max_i - ray_origin_i) * inv_dir_i;

            // Use ray.sign for branchless min/max
            let t_near = if ray.sign[i] == 0 { t0 } else { t1 };
            let t_far = if ray.sign[i] == 0 { t1 } else { t0 };

            // Track which face we're entering through
            if t_near > t_min {
                t_min = t_near;
                normal_axis = i;
                // Normal points opposite to ray direction
                normal_sign = if ray.sign[i] == 0 { -T::one() } else { T::one() };
            }

            t_max = t_max.min(t_far);

            // Early exit if no intersection
            if t_min > t_max {
                return None;
            }
        }

        // No intersection if the box is behind the ray
        if t_max < T::zero() {
            return None;
        }

        // Choose the appropriate intersection distance
        let distance = if t_min >= T::zero() { t_min } else { t_max };

        // If we're using t_max (ray starts inside the box), we need to recalculate the normal
        if t_min < T::zero() {
            // Ray starts inside the box, we're hitting the exit face
            // Recalculate which face we're exiting through
            for i in 0..3 {
                let ray_origin_i = ray.origin[i];
                let inv_dir_i = ray.inv_direction[i];
                let box_min_i = self.mins[i];
                let box_max_i = self.maxs[i];

                if !inv_dir_i.is_finite() {
                    continue;
                }

                let t0 = (box_min_i - ray_origin_i) * inv_dir_i;
                let t1 = (box_max_i - ray_origin_i) * inv_dir_i;

                let t_far = if ray.sign[i] == 0 { t1 } else { t0 };

                if (t_far - t_max).abs() < T::default_epsilon() {
                    normal_axis = i;
                    // Normal points inward (toward the box interior) when exiting
                    normal_sign = if ray.sign[i] == 0 { T::one() } else { -T::one() };
                    break;
                }
            }
        }

        // Construct the normal vector
        let mut normal_vec = Vector3::zeros();
        normal_vec[normal_axis] = normal_sign;
        let normal = Unit::new_unchecked(normal_vec);

        Some(Hit::new(distance, normal, normal))
    }
}
