use nalgebra::{Point3, RealField, Rotation3, Unit, Vector3};

use crate::{
    error::{GeometryError, Result},
    rt::Ray,
    scene::Projection,
    traits::FallibleNumeric,
};

/// Generates sampling `Ray`.
#[derive(Debug, Clone)]
pub struct Camera<T: RealField + Copy> {
    /// Observation position.
    position: Point3<T>,
    /// View target.
    look_at: Point3<T>,
    /// Camera projection mode
    projection: Projection<T>,
    /// Resolution of the image in pixels.
    resolution: [usize; 2],
}

impl<T: RealField + Copy> Camera<T> {
    /// Constructs a new `Camera`.
    ///
    /// # Errors
    ///
    /// Returns an error if either width or height in the resolution is zero.
    pub fn new(position: Point3<T>, look_at: Point3<T>, projection: Projection<T>, resolution: [usize; 2]) -> Result<Self> {
        if resolution[0] == 0 || resolution[1] == 0 {
            return Err(GeometryError::InvalidResolution {
                width: resolution[1],
                height: resolution[0],
            }
            .into());
        }

        Ok(Self {
            position,
            look_at,
            projection,
            resolution,
        })
    }

    /// Returns the resolution of the `Camera`.
    pub const fn resolution(&self) -> &[usize; 2] {
        &self.resolution
    }

    /// Generate a `Ray` for the given pixel index.
    /// A position of [0, 0] corresponds to the top-left pixel of the image.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The pixel index is out of bounds for the camera resolution
    /// - Numeric type conversions fail during ray generation
    pub fn generate_ray(&self, pixel_index: [usize; 2]) -> Result<Ray<T>> {
        if pixel_index[0] >= self.resolution[0] || pixel_index[1] >= self.resolution[1] {
            return Err(GeometryError::PixelOutOfBounds {
                row: pixel_index[0],
                col: pixel_index[1],
                res_height: self.resolution[0],
                res_width: self.resolution[1],
            }
            .into());
        }

        match self.projection {
            Projection::Perspective(fov) => self.generate_perspective_ray(pixel_index, fov),
            Projection::Orthographic(width) => self.generate_ortho_ray(pixel_index, width),
        }
    }

    /// Generate a `Ray` using a perspective projection.
    fn generate_perspective_ray(&self, pixel_index: [usize; 2], fov: T) -> Result<Ray<T>> {
        let height = T::try_from_usize(self.resolution[0])?;
        let width = T::try_from_usize(self.resolution[1])?;

        // Normalize to [-0.5, 0.5] range
        let half = T::try_from_f32(0.5)?;
        let d_row = (T::try_from_usize(pixel_index[0])? / height) - half;
        let d_col = (T::try_from_usize(pixel_index[1])? / width) - half;

        let aspect_ratio = width / height;
        let half_fov = fov * half;

        let d_theta = -d_col * half_fov;
        let d_phi = -d_row * (half_fov / aspect_ratio);

        let forward = Unit::new_normalize(self.look_at - self.position);
        let right = Unit::new_normalize(forward.cross(&Vector3::z()));
        let up = Unit::new_normalize(right.cross(&forward));

        let vertical_rotation = Rotation3::from_axis_angle(&right, d_phi);
        let lateral_rotation = Rotation3::from_axis_angle(&up, d_theta);

        let direction = lateral_rotation * vertical_rotation * forward;
        Ok(Ray::new(self.position, direction))
    }

    /// Generate a `Ray` using an orthographic projection.
    fn generate_ortho_ray(&self, pixel_index: [usize; 2], width: T) -> Result<Ray<T>> {
        let height_px = T::try_from_usize(self.resolution[0])?;
        let width_px = T::try_from_usize(self.resolution[1])?;

        // Normalize to [-0.5, 0.5] range
        let half = T::try_from_f32(0.5)?;
        let u = (T::try_from_usize(pixel_index[1])? / width_px) - half;
        let v = (T::try_from_usize(pixel_index[0])? / height_px) - half;

        // Calculate aspect ratio and viewing dimensions
        let aspect_ratio = width_px / height_px;
        let view_width = width;
        let view_height = -view_width / aspect_ratio;

        // Set up coordinate system
        let forward = Unit::new_normalize(self.look_at - self.position);
        let right = Unit::new_normalize(forward.cross(&Vector3::z()));
        let up = Unit::new_normalize(right.cross(&forward));

        // Calculate the ray origin on the viewing plane
        let horizontal_offset = right.as_ref() * (u * view_width);
        let vertical_offset = up.as_ref() * (v * view_height);

        let ray_origin = self.position + horizontal_offset + vertical_offset;

        // All rays have the same direction in orthographic projection
        Ok(Ray::new(ray_origin, forward))
    }
}
