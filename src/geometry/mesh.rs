//! Triangle mesh structure.

use nalgebra::{Point3, RealField, Unit, Vector3};
use num_traits::ToPrimitive;
use std::{borrow::Cow, fs::read_to_string, path::Path, str::FromStr};

use crate::{
    bvh::{Bvh, BvhConfig},
    geometry::{Aabb, Triangle},
    rt::{Hit, Ray},
    traits::Bounded,
};

/// Internal transient structure used to represent a `Triangle` in the `Mesh` using vertex and normal indices.
struct Face {
    /// Indices of the vertex positions.
    vertex_indices: [usize; 3],
    /// Indices of the vertex normals.
    normal_indices: [usize; 3],
}

/// Surface composed of `Triangle`s.
#[derive(Debug)]
pub struct Mesh<T: RealField + Copy> {
    /// Component `Triangle` instances.
    triangles: Vec<Triangle<T>>,
    /// `Bvh` acceleration structure.
    bvh: Bvh<T>,
}

impl<T: RealField + Copy + ToPrimitive> Mesh<T> {
    /// Construct a new `Mesh` instance.
    pub fn new(bvh_config: &BvhConfig<T>, triangles: Vec<Triangle<T>>) -> Self {
        let bvh = Bvh::new(bvh_config, &triangles);
        Self { triangles, bvh }
    }

    /// Get a reference to the `Triangle`s in this `Mesh`.
    #[must_use]
    pub fn triangles(&self) -> &[Triangle<T>] {
        &self.triangles
    }

    /// Get a reference to the `Bvh` acceleration structure.
    #[must_use]
    pub const fn bvh(&self) -> &Bvh<T> {
        &self.bvh
    }

    /// Test for an intersection between a `Ray` and the `Mesh`.
    /// Returns the closest intersection if any.
    pub fn intersect(&self, ray: &Ray<T>) -> Option<(usize, Hit<T>)> {
        self.bvh.intersect(ray, &self.triangles)
    }

    /// Test if `Ray` intersects any `Triangle` in the `Mesh` (shadow ray optimization).
    pub fn intersect_any(&self, ray: &Ray<T>, max_distance: T) -> bool {
        self.bvh.intersect_any(ray, &self.triangles, max_distance)
    }

    /// Load a `Mesh` from a wavefront (.obj) file.
    ///
    /// # Panics
    ///
    /// Panics if the .obj file is malformed or does not conform to the expected format.
    pub fn load<P: AsRef<Path>>(bvh_config: &BvhConfig<T>, path: P) -> Self
    where
        T: FromStr,
    {
        let file_string =
            read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read .obj file at path: {}", path.as_ref().display()));
        Self::from_wavefront(bvh_config, &file_string)
    }

    /// Construct a `Mesh` from a wavefront (.obj) string.
    ///
    /// # Panics
    ///
    /// Panics if the wavefront data is malformed or does not conform to the expected format.
    pub fn from_wavefront(bvh_config: &BvhConfig<T>, obj_string: &str) -> Self
    where
        T: FromStr,
    {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut faces = Vec::new();

        // Parse the OBJ file
        for line in obj_string.lines() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            match *tokens
                .first()
                .expect("Invalid .obj data: Expected at least item in each wavefront file line")
            {
                "v" => {
                    let vertex = parse_vertex_position(&tokens[1..]);
                    vertices.push(vertex);
                }
                "vn" => {
                    let normal = parse_vertex_normal(&tokens[1..]);
                    normals.push(normal);
                }
                "f" => {
                    let face = parse_face(&tokens[1..]);
                    faces.push(face);
                }
                _ => {}
            }
        }

        // Pre-build all triangles with computed data
        let triangles = faces
            .into_iter()
            .map(|face| {
                Triangle::new(
                    [
                        vertices[face.vertex_indices[0]],
                        vertices[face.vertex_indices[1]],
                        vertices[face.vertex_indices[2]],
                    ],
                    [
                        normals[face.normal_indices[0]],
                        normals[face.normal_indices[1]],
                        normals[face.normal_indices[2]],
                    ],
                )
            })
            .collect::<Vec<_>>();

        // Build BVH from the pre-built triangles
        let bvh = Bvh::new(bvh_config, &triangles);

        Self { triangles, bvh }
    }
}

impl<T: RealField + Copy> Bounded<T> for Mesh<T> {
    fn aabb(&self) -> Cow<Aabb<T>> {
        // Initialise with the first triangle's AABB
        let mut aabb = self.triangles[0].aabb().into_owned();

        // Merge AABBs of all triangles
        for triangle in &self.triangles[1..] {
            aabb = aabb.merge(&triangle.aabb());
        }

        Cow::Owned(aabb)
    }
}

// == Utility functions ==

/// Parse a vertex position from an .obj file string.
fn parse_vertex_position<T: RealField + Copy + FromStr>(coords: &[&str]) -> Point3<T> {
    assert!(coords.len() == 3, "Vertex position must have exactly 3 coordinates");
    let x = coords[0]
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Invalid x coordinate for vertex position"));
    let y = coords[1]
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Invalid y coordinate for vertex position"));
    let z = coords[2]
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Invalid z coordinate for vertex position"));
    Point3::new(x, y, z)
}

/// Parse a vertex normal from an .obj file string.
fn parse_vertex_normal<T: RealField + Copy + FromStr>(coords: &[&str]) -> Unit<Vector3<T>> {
    assert!(coords.len() == 3, "Vertex normal must have exactly 3 coordinates");
    let xn = coords[0]
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Invalid x coordinate for vertex normal"));
    let yn = coords[1]
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Invalid y coordinate for vertex normal"));
    let zn = coords[2]
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Invalid z coordinate for vertex normal"));
    Unit::new_normalize(Vector3::new(xn, yn, zn))
}

/// Parse a face from an .obj file string.
fn parse_face(tokens: &[&str]) -> Face {
    assert!(
        tokens.len() == 3,
        "Face must have exactly 3 vertex indices (must be triangular face mesh)"
    );

    let mut vertex_indices = [0; 3];
    let mut normal_indices = [0; 3];

    for (i, token) in tokens.iter().enumerate() {
        vertex_indices[i] = token
            .split('/')
            .next()
            .expect("Face must specify a vertex position index")
            .parse::<usize>()
            .expect("Invalid face vertex position index!")
            - 1;
        normal_indices[i] = token
            .split('/')
            .next_back()
            .expect("Face must specify a vertex normal index")
            .parse::<usize>()
            .expect("Invalid face vertex normal index")
            - 1;
    }

    Face {
        vertex_indices,
        normal_indices,
    }
}
