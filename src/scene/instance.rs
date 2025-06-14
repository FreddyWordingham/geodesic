//! Mesh instance structure.

use nalgebra::{Matrix3, Matrix4, RealField, Unit};
use num_traits::ToPrimitive;

use crate::{
    error::{Result, TransformationError},
    geometry::{Aabb, Mesh},
    rt::{Hit, Ray},
    traits::{Bounded, Traceable},
};

/// `Mesh` instance allowing for transformations without copying the original data.
#[derive(Debug)]
pub struct Instance<'a, T: RealField + Copy> {
    /// Reference to `Mesh` data.
    mesh: &'a Mesh<T>,
    /// World-to-object transformation matrix.
    world_to_object: Matrix4<T>,
    /// Object-to-world transformation matrix.
    object_to_world: Matrix4<T>,
    /// Transformed bounding box in world space.
    world_aabb: Aabb<T>,
    /// Pre-computed normal transformation matrix (inverse transpose of upper 3x3)
    normal_transform: Matrix3<T>,
}

impl<'a, T: RealField + Copy + ToPrimitive> Instance<'a, T> {
    /// Construct a new `Mesh` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transformation matrix is not invertible
    /// - Bounding box transformation fails
    pub fn new(mesh: &'a Mesh<T>, transform: Matrix4<T>) -> Result<Self> {
        let world_to_object = transform.try_inverse().ok_or(TransformationError::NonInvertibleMatrix)?;

        let object_to_world = transform;
        let world_aabb = mesh.aabb()?.transform(&object_to_world)?;

        let upper_3x3 = world_to_object.fixed_view::<3, 3>(0, 0);
        let normal_transform = upper_3x3.transpose();

        Ok(Self {
            mesh,
            world_to_object,
            object_to_world,
            world_aabb,
            normal_transform,
        })
    }

    /// Get a reference to the underlying `Mesh`.
    pub const fn mesh(&self) -> &Mesh<T> {
        self.mesh
    }

    /// Get the world-space `Aabb`.
    pub const fn world_aabb(&self) -> &Aabb<T> {
        &self.world_aabb
    }

    /// Transform a `Ray` from world space to object space.
    fn transform_ray_to_object_space(&self, ray: &Ray<T>) -> Ray<T> {
        // Transform origin using the built-in transform_point method
        let object_origin = self.world_to_object.transform_point(&ray.origin);

        // Transform direction as a vector (homogeneous coordinate w=0)
        let object_direction_vector = self.world_to_object.transform_vector(&ray.direction);

        // Normalize the direction
        let object_direction = Unit::new_normalize(object_direction_vector);

        Ray::new(object_origin, object_direction)
    }

    /// Transform a `Hit` from object space to world space.
    fn transform_hit_to_world_space(&self, hit: &mut Hit<T>, world_ray: &Ray<T>, object_ray: &Ray<T>) {
        // Transform geometric normal
        let world_geometric_normal_vector = self.normal_transform * hit.geometric_normal.as_ref();
        hit.geometric_normal = Unit::new_normalize(world_geometric_normal_vector);

        // Transform interpolated normal
        let world_interpolated_normal_vector = self.normal_transform * hit.interpolated_normal.as_ref();
        hit.interpolated_normal = Unit::new_normalize(world_interpolated_normal_vector);

        // Transform the distance from object space to world space. The hit distance is along the object-space ray, but we need it along the world-space ray
        // Calculate the actual world-space intersection point
        let object_hit_point = object_ray.origin + object_ray.direction.scale(hit.distance);
        let world_hit_point = self.object_to_world.transform_point(&object_hit_point);

        // Calculate the distance along the world ray to reach this point
        let to_hit = world_hit_point - world_ray.origin;
        hit.distance = to_hit.dot(&world_ray.direction);
    }
}

impl<T: RealField + Copy + ToPrimitive> Traceable<T> for Instance<'_, T> {
    fn intersect(&self, ray: &Ray<T>) -> Result<Option<Hit<T>>> {
        // Transform ray to object space
        let object_ray = self.transform_ray_to_object_space(ray);

        // Intersect with the mesh in object space
        (self.mesh.intersect(&object_ray)?).map_or(Ok(None), |mut hit| {
            // Transform hit back to world space
            self.transform_hit_to_world_space(&mut hit, ray, &object_ray);
            // The object_index from the mesh is the triangle index within the mesh
            // This is preserved through the transformation
            Ok(Some(hit))
        })
    }

    fn intersect_any(&self, ray: &Ray<T>, max_distance: T) -> Result<bool> {
        // Transform ray to object space
        let object_ray = self.transform_ray_to_object_space(ray);

        // Transform max_distance from world space to object space
        // We need to account for how the transformation affects distances along the ray
        let world_endpoint = ray.origin + ray.direction.scale(max_distance);
        let object_endpoint = self.world_to_object.transform_point(&world_endpoint);
        let object_max_distance = (object_endpoint - object_ray.origin).norm();
        self.mesh.intersect_any(&object_ray, object_max_distance)
    }
}
