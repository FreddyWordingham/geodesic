use nalgebra::{Point3, RealField, Rotation3, Unit, Vector3};

use crate::prelude::*;

#[derive(Clone)]
pub enum CameraType<T: RealField + Copy> {
    Perspective(T),
    Orthographic(T),
}

/// Generates sampling rays to form an image.
#[derive(Clone)]
pub struct Camera<T: RealField + Copy> {
    /// Observation position.
    position: Point3<T>,
    /// View target.
    look_at: Point3<T>,
    /// Camera type
    camera_type: CameraType<T>,
    /// Resolution of the image in pixels.
    resolution: [usize; 2],
}

impl<T: RealField + Copy> Camera<T> {
    /// Constructs a new `Camera`.
    pub fn new(position: Point3<T>, look_at: Point3<T>, camera_type: CameraType<T>, resolution: [usize; 2]) -> Self {
        debug_assert!(resolution[0] > 0, "Resolution height must be positive");
        debug_assert!(resolution[1] > 0, "Resolution width must be positive");

        Self {
            position,
            look_at,
            camera_type,
            resolution,
        }
    }

    /// Returns the resolution of the camera.
    pub fn resolution(&self) -> &[usize; 2] {
        &self.resolution
    }

    pub fn generate_ray(&self, pixel_index: [usize; 2]) -> Ray<T> {
        match self.camera_type {
            CameraType::Perspective(fov) => self.generate_perspective_ray(pixel_index, fov),
            CameraType::Orthographic(width) => self.generate_ortho_ray(pixel_index, width),
        }
    }

    fn generate_perspective_ray(&self, pixel_index: [usize; 2], fov: T) -> Ray<T> {
        debug_assert!(pixel_index[0] < self.resolution[0], "Row index out of bounds");
        debug_assert!(pixel_index[1] < self.resolution[1], "Column index out of bounds");

        let height = T::from_usize(self.resolution[0]).unwrap();
        let width = T::from_usize(self.resolution[1]).unwrap();

        // Normalize to [-0.5, 0.5] range
        let d_row = (T::from_usize(pixel_index[0]).unwrap() / height.clone()) - T::from_f32(0.5).unwrap();
        let d_col = (T::from_usize(pixel_index[1]).unwrap() / width.clone()) - T::from_f32(0.5).unwrap();

        let aspect_ratio = width / height;
        let half_fov = fov * T::from_f32(0.5).unwrap();

        let d_theta = -d_col * half_fov.clone();
        let d_phi = -d_row * (half_fov / aspect_ratio);

        let forward = Unit::new_normalize(&self.look_at - &self.position);
        let right = Unit::new_normalize(forward.cross(&Vector3::z()));
        let up = Unit::new_normalize(right.cross(&forward));

        let vertical_rotation = Rotation3::from_axis_angle(&right, d_phi);
        let lateral_rotation = Rotation3::from_axis_angle(&up, d_theta);

        let direction = lateral_rotation * vertical_rotation * forward;
        Ray::new(self.position.clone(), direction)
    }

    fn generate_ortho_ray(&self, pixel_index: [usize; 2], width: T) -> Ray<T> {
        debug_assert!(pixel_index[0] < self.resolution[0], "Row index out of bounds");
        debug_assert!(pixel_index[1] < self.resolution[1], "Column index out of bounds");

        let height_px = T::from_usize(self.resolution[0]).unwrap();
        let width_px = T::from_usize(self.resolution[1]).unwrap();

        // Normalize to [-0.5, 0.5] range
        let u = (T::from_usize(pixel_index[1]).unwrap() / width_px) - T::from_f32(0.5).unwrap();
        let v = (T::from_usize(pixel_index[0]).unwrap() / height_px) - T::from_f32(0.5).unwrap();

        // Calculate aspect ratio and viewing dimensions
        let aspect_ratio = width_px / height_px;
        let view_width = width;
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
