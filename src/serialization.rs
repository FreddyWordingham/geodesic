use nalgebra::{Matrix4, Point3, RealField, Rotation3, Translation3, Unit, Vector3};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

use crate::{
    assets::Assets,
    bvh_config::BvhConfig,
    mesh::Mesh,
    plane::Plane,
    prelude::{Instance, SceneObject},
    scene::Scene,
    sphere::Sphere,
    triangle::Triangle,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAssets<T: RealField + Copy + ToPrimitive> {
    pub bvh_config: Option<BvhConfig<T>>,
    pub meshes: Vec<(String, PathBuf)>, // (identifier, file path)
}

impl<T: RealField + Copy + ToPrimitive + FromStr> SerializedAssets<T> {
    pub fn build(self) -> Assets<T> {
        let bvh_config = self.bvh_config.unwrap_or_else(|| BvhConfig::default());
        let mut assets = Assets::empty(bvh_config.clone());
        for (name, path) in self.meshes {
            let mesh = Mesh::load(&bvh_config, path);
            assets = assets.add_mesh(&name, mesh);
        }
        assets
    }
}

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
    Instance(String, Option<SerialzedTransform<T>>), // Mesh identifier and optional transformation
}

impl<T: RealField + Copy + ToPrimitive> SerializedSceneObject<T> {
    pub fn build<'a>(self, assets: &'a Assets<T>) -> SceneObject<'a, T> {
        match self {
            SerializedSceneObject::Sphere(center, radius) => SceneObject::Sphere(Sphere::new(center.into(), radius)),
            SerializedSceneObject::Plane(point, normal) => {
                let point = Point3::new(point[0], point[1], point[2]);
                let normal = Unit::new_normalize(Vector3::new(normal[0], normal[1], normal[2]));
                SceneObject::Plane(Plane::new(point, normal))
            }
            SerializedSceneObject::Triangle(positions, normals) => SceneObject::Triangle(Triangle::new(
                positions.map(|p| Point3::new(p[0], p[1], p[2])),
                normals.map(|n| Unit::new_normalize(Vector3::new(n[0], n[1], n[2]))),
            )),
            SerializedSceneObject::Instance(mesh_id, transform) => {
                let mesh = assets.meshes.get(&mesh_id).unwrap();
                let transform = transform.and_then(|t| t.build()).unwrap_or_else(|| Matrix4::identity());
                SceneObject::Instance(Instance::new(mesh, transform))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialzedTransform<T: RealField + Copy> {
    pub translation: Option<[T; 3]>, // Translation vector
    pub rotation: Option<[T; 3]>,    // Euler angles
    pub scale: Option<T>,            // Uniform scale
}

impl<T: RealField + Copy> SerialzedTransform<T> {
    pub fn build(self) -> Option<Matrix4<T>> {
        let translation = if let Some(translation) = self.translation {
            Translation3::new(translation[0], translation[1], translation[2])
        } else {
            Translation3::identity()
        };

        let rotation = if let Some(rotation) = self.rotation {
            // Apply rotation (assuming Euler angles in radians)
            Rotation3::from_euler_angles(rotation[0], rotation[1], rotation[2])
        } else {
            Rotation3::identity()
        };

        let scale_matrix = if let Some(scale) = self.scale {
            Matrix4::new_scaling(scale)
        } else {
            Matrix4::identity()
        };

        // Combine transformations: Translation * Rotation * Scale
        Some(translation.to_homogeneous() * rotation.to_homogeneous() * scale_matrix)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedScene<T: RealField + Copy> {
    pub objects: Vec<SerializedSceneObject<T>>,
}

impl<T: RealField + Copy + ToPrimitive> SerializedScene<T> {
    pub fn build(self, assets: &Assets<T>) -> Scene<'_, T> {
        let objects: Vec<SceneObject<T>> = self.objects.into_iter().map(|obj| obj.build(assets)).collect();
        // Ensure we have at least one object to create a valid scene
        assert!(!objects.is_empty(), "Scene must contain at least one object");
        Scene::new(&assets.bvh_config, objects)
    }
}
