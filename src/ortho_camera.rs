use nalgebra::{Point3, RealField, Unit, Vector3, distance};

use crate::ray::Ray;

/// Generates sampling rays for orthographic projection.
#[derive(Clone)]
pub struct OrthoCamera<T: RealField + Copy> {
    /// Observation position (center of the orthographic viewing plane).
    position: Point3<T>,
    /// View target.
    look_at: Point3<T>,
    /// Width of the orthographic viewing volume.
    width: T,
    /// Resolution of the image in pixels.
    resolution: [usize; 2],
}

impl<T: RealField + Copy> OrthoCamera<T> {
    /// Constructs a new `OrthoCamera`.
    pub fn new(position: Point3<T>, look_at: Point3<T>, width: T, resolution: [usize; 2]) -> Self {
        debug_assert!(
            distance(&position, &look_at) > T::zero(),
            "Camera position and look-at point must be distinct"
        );
        debug_assert!(width > T::zero(), "Viewing width must be positive");
        debug_assert!(resolution[0] > 0, "Resolution height must be positive");
        debug_assert!(resolution[1] > 0, "Resolution width must be positive");

        Self {
            position,
            look_at,
            width,
            resolution,
        }
    }

    pub fn generate_ray(&self, pixel_index: [usize; 2]) -> Ray<T> {
        debug_assert!(pixel_index[0] < self.resolution[0], "Row index out of bounds");
        debug_assert!(pixel_index[1] < self.resolution[1], "Column index out of bounds");

        let height_px = T::from_usize(self.resolution[0]).unwrap();
        let width_px = T::from_usize(self.resolution[1]).unwrap();

        // Normalize to [-0.5, 0.5] range
        let u = (T::from_usize(pixel_index[1]).unwrap() / width_px) - T::from_f32(0.5).unwrap();
        let v = (T::from_usize(pixel_index[0]).unwrap() / height_px) - T::from_f32(0.5).unwrap();

        // Calculate aspect ratio and viewing dimensions
        let aspect_ratio = width_px / height_px;
        let view_width = self.width;
        let view_height = -view_width / aspect_ratio;

        // Set up coordinate system
        let forward = Unit::new_normalize(&self.look_at - &self.position);
        let right = Unit::new_normalize(forward.cross(&Vector3::z()));
        let up = Unit::new_normalize(right.cross(&forward));

        // Calculate the ray origin on the viewing plane
        let horizontal_offset = right.as_ref() * (u * view_width);
        let vertical_offset = up.as_ref() * (v * view_height);

        let ray_origin = self.position + horizontal_offset + vertical_offset;

        // All rays have the same direction in orthographic projection
        Ray::new(ray_origin, forward)
    }
}
