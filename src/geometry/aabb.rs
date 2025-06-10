//! Axis-aligned bounding box structure.

use nalgebra::{Matrix4, Point3, RealField};
use std::borrow::Cow;

use crate::{
    error::{GeometryError, Result},
    rt::Ray,
    traits::{Bounded, FallibleNumeric},
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
    pub fn new(mins: Point3<T>, maxs: Point3<T>) -> Result<Self> {
        if mins.x > maxs.x || mins.y > maxs.y || mins.z > maxs.z {
            return Err(GeometryError::InvalidAabbBounds {
                min_x: format!("{:?}", mins.x),
                min_y: format!("{:?}", mins.y),
                min_z: format!("{:?}", mins.z),
                max_x: format!("{:?}", maxs.x),
                max_y: format!("{:?}", maxs.y),
                max_z: format!("{:?}", maxs.z),
            }
            .into());
        }
        Ok(Self { mins, maxs })
    }

    /// Create an 'empty' `Aabb` with extreme bounds.
    ///
    /// # Panics
    ///
    /// In practice this method will never panic.
    pub fn empty() -> Result<Self> {
        let min_value = T::try_min_value()?;
        let max_value = T::try_max_value()?;
        Ok(Self {
            mins: Point3::new(max_value, max_value, max_value),
            maxs: Point3::new(min_value, min_value, min_value),
        })
    }

    /// Calculate the center of the `Aabb`.
    ///
    /// # Panics
    ///
    /// In practice this method will never panic.
    pub fn centre(&self) -> Result<Point3<T>> {
        let two = T::try_from_u8(2)?;
        Ok(Point3::new(
            (self.mins.x + self.maxs.x) / two,
            (self.mins.y + self.maxs.y) / two,
            (self.mins.z + self.maxs.z) / two,
        ))
    }

    /// Calculate the surface area of an `Aabb`.
    ///
    /// # Panics
    ///
    /// In practice this method will never panic.
    pub fn surface_area(&self) -> Result<T> {
        let extent = [
            self.maxs[0] - self.mins[0],
            self.maxs[1] - self.mins[1],
            self.maxs[2] - self.mins[2],
        ];
        let two = T::try_from_u8(2)?;
        Ok(two * ((extent[0] * extent[1]) + (extent[1] * extent[2]) + (extent[2] * extent[0])))
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
    pub fn merge(&self, other: &Self) -> Result<Self> {
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
    pub fn transform(&self, transform: &Matrix4<T>) -> Result<Self> {
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
    pub fn intersect_any(&self, ray: &Ray<T>) -> Result<bool> {
        let mut t_min = T::zero();
        let mut t_max = T::try_max_value()?;

        // Use pre-computed inverse directions from Ray struct
        for i in 0..3 {
            let ray_origin_i = ray.origin[i];
            let inv_dir_i = ray.inv_direction[i];
            let box_min_i = self.mins[i];
            let box_max_i = self.maxs[i];

            // Check for parallel ray (inv_direction will be inf/-inf)
            if !inv_dir_i.is_finite() {
                if ray_origin_i < box_min_i || ray_origin_i > box_max_i {
                    return Ok(false);
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
                return Ok(false);
            }
        }

        // If the maximum distance is negative, the box is behind the ray
        Ok(match t_max.partial_cmp(&T::zero()) {
            Some(std::cmp::Ordering::Less) => false,
            Some(_) => true,
            None => unimplemented!("t_max is NaN, cannot determine intersection"),
        })
    }

    /// Test for an intersection between a `Ray` and the `Aabb`.
    ///
    /// # Panics
    ///
    /// In practice this method will never panic.
    pub fn intersect_distance(&self, ray: &Ray<T>) -> Result<Option<T>> {
        let mut t_min = T::zero();
        let mut t_max = T::try_max_value()?;

        // Use pre-computed inverse directions from Ray struct
        for i in 0..3 {
            let ray_origin_i = ray.origin[i];
            let inv_dir_i = ray.inv_direction[i];
            let box_min_i = self.mins[i];
            let box_max_i = self.maxs[i];

            // Check for parallel ray (inv_direction will be inf/-inf)
            if !inv_dir_i.is_finite() {
                if ray_origin_i < box_min_i || ray_origin_i > box_max_i {
                    return Ok(None);
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
                return Ok(None);
            }
        }

        if t_max < T::zero() {
            return Ok(None);
        }

        Ok(Some(if t_min >= T::zero() { t_min } else { t_max }))
    }
}

impl<T: RealField + Copy> Bounded<T> for Aabb<T> {
    fn aabb(&self) -> Result<Cow<Self>> {
        Ok(Cow::Borrowed(self))
    }
}
