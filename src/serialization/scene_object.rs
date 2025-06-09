use nalgebra::{Matrix4, Point3, RealField, Unit, Vector3};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::{
    geometry::{Plane, Sphere, Triangle},
    scene::{Assets, Instance, SceneObject},
    serialization::SerializedTransform,
};

/// Enumeration of all `Traceable` objects that can be added to a `Scene`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedSceneObject<T: RealField + Copy> {
    /// A sphere primitive.
    Sphere([T; 3], T), // Center and radius
    /// An infinite plane primitive.
    Plane([T; 3], [T; 3]), // Point and normal vectors
    /// A triangle primitive.
    Triangle([[T; 3]; 3], [[T; 3]; 3]), // Vertex positions and normals
    /// A mesh instance with transformation.
    Instance(String, Option<SerializedTransform<T>>), // Mesh identifier and optional transformation
}

impl<T: RealField + Copy + ToPrimitive> SerializedSceneObject<T> {
    /// Construct a `SceneObject` instance.
    pub fn build<'a>(self, assets: &'a Assets<T>) -> SceneObject<'a, T> {
        match self {
            Self::Sphere(center, radius) => SceneObject::Sphere(Sphere::new(center.into(), radius)),
            Self::Plane(point, normal) => {
                let point = Point3::new(point[0], point[1], point[2]);
                let normal = Unit::new_normalize(Vector3::new(normal[0], normal[1], normal[2]));
                SceneObject::Plane(Plane::new(point, normal))
            }
            Self::Triangle(positions, normals) => SceneObject::Triangle(Triangle::new(
                positions.map(|p| Point3::new(p[0], p[1], p[2])),
                normals.map(|n| Unit::new_normalize(Vector3::new(n[0], n[1], n[2]))),
            )),
            Self::Instance(mesh_id, transform) => {
                let mesh = assets.meshes.get(&mesh_id).unwrap();
                let transform = transform.map(SerializedTransform::build).unwrap_or_else(Matrix4::identity);
                SceneObject::Instance(Instance::new(mesh, transform))
            }
        }
    }
}
