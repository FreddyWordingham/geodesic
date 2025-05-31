//! Mesh instance structure.

use nalgebra::{Matrix4, RealField, Unit};
use num_traits::ToPrimitive;

use crate::{aabb::Aabb, bounded::Bounded, hit::Hit, mesh::Mesh, ray::Ray};

/// Triangle mesh instance.
pub struct Instance<'a, T: RealField + Copy> {
    /// Reference to `Mesh` data.
    mesh: &'a Mesh<T>,
    /// World-to-object transformation matrix.
    world_to_object: Matrix4<T>,
    // /// Object-to-world transformation matrix.
    // object_to_world: Matrix4<T>,
    /// Transformed bounding box in world space.
    world_aabb: Aabb<T>,
}

impl<'a, T: RealField + Copy + ToPrimitive> Instance<'a, T> {
    /// Construct a new `Mesh` instance.
    pub fn new(mesh: &'a Mesh<T>, transform: Matrix4<T>) -> Self {
        let world_to_object = transform.try_inverse().expect("Transformation matrix must be invertible");
        let object_to_world = transform;
        let world_aabb = mesh.aabb().transform(&object_to_world);

        Self {
            mesh,
            world_to_object,
            // object_to_world,
            world_aabb,
        }
    }

    /// Get a reference to the underlying `Mesh`.
    pub fn mesh(&self) -> &Mesh<T> {
        self.mesh
    }

    /// Get the world-space `Aabb`.
    pub fn world_aabb(&self) -> &Aabb<T> {
        &self.world_aabb
    }

    /// Test for an intersection between a `Ray` and the `Instance`.
    /// Transforms the ray to object space, performs intersection, then transforms result back.
    pub fn intersect(&self, ray: &Ray<T>) -> Option<(usize, Hit<T>)> {
        // Transform ray to object space
        let object_ray = self.transform_ray_to_object_space(ray)?;

        // Intersect with the mesh in object space
        let (triangle_index, mut hit) = self.mesh.intersect(&object_ray)?;

        // Transform hit back to world space
        self.transform_hit_to_world_space(&mut hit);

        Some((triangle_index, hit))
    }

    /// Test if ray intersects any triangle in the instance (shadow ray optimization).
    pub fn intersect_any(&self, ray: &Ray<T>, max_distance: T) -> bool {
        // Transform ray to object space
        if let Some(object_ray) = self.transform_ray_to_object_space(ray) {
            // We need to transform the max_distance as well
            // This is an approximation - for precise results, we'd need more complex math
            self.mesh.intersect_any(&object_ray, max_distance)
        } else {
            false
        }
    }

    /// Transform a ray from world space to object space.
    fn transform_ray_to_object_space(&self, ray: &Ray<T>) -> Option<Ray<T>> {
        // Transform origin using the built-in transform_point method
        let object_origin = self.world_to_object.transform_point(&ray.origin);

        // Transform direction as a vector (homogeneous coordinate w=0)
        let object_direction_vector = self.world_to_object.transform_vector(&ray.direction);

        // Normalize the direction
        let object_direction = Unit::new_normalize(object_direction_vector);

        Some(Ray::new(object_origin, object_direction))
    }

    /// Transform a hit from object space to world space.
    fn transform_hit_to_world_space(&self, hit: &mut Hit<T>) {
        // For normal transformation, we need the inverse transpose of the upper 3x3 matrix
        // Since we're using Matrix4, we need to extract the 3x3 part and invert-transpose it
        let upper_3x3 = self.world_to_object.fixed_view::<3, 3>(0, 0);

        // For normal transformation: use (M^-1)^T = (M^T)^-1
        // Since we already have M^-1 (world_to_object), we need its transpose
        let normal_transform = upper_3x3.transpose();

        // Transform geometric normal
        let world_geometric_normal_vector = normal_transform * hit.geometric_normal.as_ref();
        hit.geometric_normal = Unit::new_normalize(world_geometric_normal_vector);

        // Transform interpolated normal
        let world_interpolated_normal_vector = normal_transform * hit.interpolated_normal.as_ref();
        hit.interpolated_normal = Unit::new_normalize(world_interpolated_normal_vector);

        // Note: The distance remains the same as it's measured along the original ray
        // If you need the actual world-space distance, you'd need to scale by the transform
    }
}
