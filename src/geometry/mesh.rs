//! Triangle mesh structure.

use nalgebra::{Point3, RealField, Unit, Vector3};
use num_traits::ToPrimitive;
use std::{borrow::Cow, fs::read_to_string, path::Path, str::FromStr};

use crate::{
    bvh::{Bvh, BvhConfig},
    error::{FileParsingError, Result},
    geometry::{Aabb, Triangle},
    rt::{Hit, Ray},
    traits::{Bounded, Traceable},
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
    pub fn new(bvh_config: &BvhConfig<T>, triangles: Vec<Triangle<T>>) -> Result<Self> {
        let bvh = Bvh::new(bvh_config, &triangles)?;
        Ok(Self { triangles, bvh })
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

    /// Load a `Mesh` from a wavefront (.obj) file.
    pub fn load<P: AsRef<Path>>(bvh_config: &BvhConfig<T>, path: P) -> Result<Self>
    where
        T: FromStr,
    {
        let file_string = read_to_string(&path).map_err(|_| FileParsingError::FileNotFound {
            path: path.as_ref().display().to_string(),
        })?;

        Self::from_wavefront(bvh_config, &file_string)
    }

    /// Construct a `Mesh` from a wavefront (.obj) string.
    pub fn from_wavefront(bvh_config: &BvhConfig<T>, obj_string: &str) -> Result<Self>
    where
        T: FromStr,
    {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut faces = Vec::new();

        for (line_num, line) in obj_string.lines().enumerate() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            match tokens[0] {
                "v" => {
                    if tokens.len() < 4 {
                        return Err(FileParsingError::MissingVertexPosition { line: line_num + 1 }.into());
                    }
                    let vertex = parse_vertex_position(&tokens[1..], line_num + 1)?;
                    vertices.push(vertex);
                }
                "vn" => {
                    if tokens.len() < 4 {
                        return Err(FileParsingError::MissingVertexNormal { line: line_num + 1 }.into());
                    }
                    let normal = parse_vertex_normal(&tokens[1..], line_num + 1)?;
                    normals.push(normal);
                }
                "f" => {
                    if tokens.len() < 4 {
                        return Err(FileParsingError::InvalidFaceData {
                            line: line_num + 1,
                            message: "Face must have at least 3 vertices".to_string(),
                        }
                        .into());
                    }
                    let face = parse_face(&tokens[1..], line_num + 1)?;
                    faces.push(face);
                }
                _ => {}
            }
        }

        if vertices.is_empty() {
            return Err(FileParsingError::InvalidObjFormat {
                message: "No vertices found in OBJ file".to_string(),
            }
            .into());
        }

        if faces.is_empty() {
            return Err(FileParsingError::InvalidObjFormat {
                message: "No faces found in OBJ file".to_string(),
            }
            .into());
        }

        let triangles = faces
            .into_iter()
            .map(|face| {
                if face.vertex_indices.iter().any(|&i| i >= vertices.len()) {
                    return Err(FileParsingError::InvalidFaceData {
                        line: 0, // We've lost line info here, could be improved
                        message: "Face references non-existent vertex".to_string(),
                    }
                    .into());
                }

                if face.normal_indices.iter().any(|&i| i >= normals.len()) {
                    return Err(FileParsingError::InvalidFaceData {
                        line: 0,
                        message: "Face references non-existent normal".to_string(),
                    }
                    .into());
                }

                Ok(Triangle::new(
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
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        let bvh = Bvh::new(bvh_config, &triangles)?;
        Ok(Self { triangles, bvh })
    }
}

impl<T: RealField + Copy> Bounded<T> for Mesh<T> {
    fn aabb(&self) -> Result<Cow<Aabb<T>>> {
        // Initialise with the first triangle's AABB
        let mut aabb = self.triangles[0].aabb()?.into_owned();

        // Merge AABBs of all triangles
        for triangle in &self.triangles[1..] {
            aabb = aabb.merge(&triangle.aabb()?.into_owned())?;
        }

        Ok(Cow::Owned(aabb))
    }
}

impl<T: RealField + Copy + ToPrimitive> Traceable<T> for Mesh<T> {
    fn intersect(&self, ray: &Ray<T>) -> Result<Option<Hit<T>>> {
        self.bvh.intersect(ray, &self.triangles).map(|opt| {
            opt.map(|(triangle_index, mut hit)| {
                hit.index = triangle_index;
                hit
            })
        })
    }

    fn intersect_any(&self, ray: &Ray<T>, max_distance: T) -> Result<bool> {
        self.bvh.intersect_any(ray, &self.triangles, max_distance)
    }
}

// == Utility functions ==

/// Parse a vertex position from an .obj file string.
fn parse_vertex_position<T: RealField + Copy + FromStr>(coords: &[&str], line: usize) -> Result<Point3<T>> {
    if coords.len() != 3 {
        return Err(FileParsingError::InvalidFaceData {
            line,
            message: "Vertex position must have exactly 3 coordinates".to_string(),
        }
        .into());
    }

    let parse_coord = |coord: &str| -> Result<T> {
        coord.parse::<T>().map_err(|_| {
            FileParsingError::InvalidCoordinate {
                value: coord.to_string(),
                line,
            }
            .into()
        })
    };

    let x = parse_coord(coords[0])?;
    let y = parse_coord(coords[1])?;
    let z = parse_coord(coords[2])?;

    Ok(Point3::new(x, y, z))
}

/// Parse a vertex normal from an .obj file string.
fn parse_vertex_normal<T: RealField + Copy + FromStr>(coords: &[&str], line: usize) -> Result<Unit<Vector3<T>>> {
    if coords.len() != 3 {
        return Err(FileParsingError::InvalidFaceData {
            line,
            message: "Vertex normal must have exactly 3 coordinates".to_string(),
        }
        .into());
    }

    let parse_coord = |coord: &str| -> Result<T> {
        coord.parse::<T>().map_err(|_| {
            FileParsingError::InvalidCoordinate {
                value: coord.to_string(),
                line,
            }
            .into()
        })
    };

    let xn = parse_coord(coords[0])?;
    let yn = parse_coord(coords[1])?;
    let zn = parse_coord(coords[2])?;

    Ok(Unit::new_normalize(Vector3::new(xn, yn, zn)))
}

/// Parse a face from an .obj file string.
fn parse_face(tokens: &[&str], line: usize) -> Result<Face> {
    if tokens.len() != 3 {
        return Err(FileParsingError::InvalidFaceData {
            line,
            message: "Face must have exactly 3 vertex indices (triangular faces only)".to_string(),
        }
        .into());
    }

    let mut vertex_indices = [0; 3];
    let mut normal_indices = [0; 3];

    for (i, token) in tokens.iter().enumerate() {
        let parts: Vec<&str> = token.split('/').collect();

        if parts.is_empty() {
            return Err(FileParsingError::InvalidFaceData {
                line,
                message: "Face must specify vertex indices".to_string(),
            }
            .into());
        }

        vertex_indices[i] = parts[0]
            .parse::<usize>()
            .map_err(|_| FileParsingError::InvalidFaceData {
                line,
                message: format!("Invalid vertex index: {}", parts[0]),
            })?
            .saturating_sub(1); // OBJ indices are 1-based

        if parts.len() < 3 {
            return Err(FileParsingError::InvalidFaceData {
                line,
                message: "Face must specify normal indices".to_string(),
            }
            .into());
        }

        normal_indices[i] = parts[2]
            .parse::<usize>()
            .map_err(|_| FileParsingError::InvalidFaceData {
                line,
                message: format!("Invalid normal index: {}", parts[2]),
            })?
            .saturating_sub(1); // OBJ indices are 1-based
    }

    Ok(Face {
        vertex_indices,
        normal_indices,
    })
}
