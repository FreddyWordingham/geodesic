//! Bounding Volume Hierarchy node structure with Surface Area Heuristic.

use nalgebra::RealField;
use num_traits::ToPrimitive;
use std::borrow::Cow;

use crate::{
    bvh::{BvhBuilder, BvhConfig},
    error::{BvhError, Result},
    geometry::Aabb,
    rt::{Hit, Ray},
    traits::{Bounded, FallibleNumeric, Traceable},
};

/// Bounding volume hierarchy node.
#[derive(Debug, Clone)]
pub struct BvhNode<T: RealField + Copy> {
    /// Bounding box.
    pub aabb: Aabb<T>,
    /// Left child node index. Right child node index is `left_child + 1`.
    pub left_child: usize,
    /// Number of objects contained in this node.
    pub count: usize,
}

/// Bounding Volume Hierarchy (BVH) structure used to accelerate ray tracing by reducing the number of intersection tests required.
#[derive(Debug)]
pub struct Bvh<T: RealField + Copy> {
    /// Indices of objects contained in this node.
    indices: Vec<usize>,
    /// List of nodes.
    nodes: Vec<BvhNode<T>>,
    /// Depth of the tree.
    depth: usize,
}

impl<T: RealField + Copy + ToPrimitive> Bvh<T> {
    /// Construct a new `Bvh` instance using a builder and a collection of `Bounded` shapes.
    pub fn new<B: Bounded<T>>(config: &BvhConfig<T>, shapes: &[B]) -> Result<Self> {
        BvhBuilder::new(config).build(shapes)
    }

    /// Construct a new `Bvh` instance directly.
    ///
    /// # Panics
    ///
    /// Panics if `indices` or `nodes` are empty.
    pub fn construct_directly(indices: Vec<usize>, nodes: Vec<BvhNode<T>>, depth: usize) -> Result<Self> {
        if indices.is_empty() {
            return Err(BvhError::EmptyGeometry.into());
        }
        if nodes.is_empty() {
            return Err(BvhError::EmptyNodes.into());
        }
        Ok(Self { indices, nodes, depth })
    }

    /// Get the depth of the `Bvh` tree.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Test for intersections between a `Ray` and geometries in the `Bvh`.
    /// Returns the closest intersection if any.
    pub fn intersect<B>(&self, ray: &Ray<T>, shapes: &[B]) -> Result<Option<(usize, Hit<T>)>>
    where
        B: Bounded<T> + Traceable<T>,
    {
        self.intersect_recursive(ray, shapes, 0)
    }

    /// Test if a `Ray` intersects any geometry in the `Bvh` (shadow ray optimization).
    pub fn intersect_any<B>(&self, ray: &Ray<T>, shapes: &[B], max_distance: T) -> Result<bool>
    where
        B: Bounded<T> + Traceable<T>,
    {
        self.intersect_any_recursive(ray, shapes, 0, max_distance)
    }

    /// Recursive helper for `Bvh` traversal.
    fn intersect_recursive<B>(&self, ray: &Ray<T>, shapes: &[B], node_index: usize) -> Result<Option<(usize, Hit<T>)>>
    where
        B: Bounded<T> + Traceable<T>,
    {
        if node_index >= self.nodes.len() {
            return Ok(None);
        }

        let node = &self.nodes[node_index];

        // Test ray against node's bounding box
        if !node.aabb.intersect_any(ray)? {
            return Ok(None);
        }

        // Leaf node - test against primitives
        if node.count > 0 {
            let mut closest_hit: Option<(usize, Hit<T>)> = None;
            let mut closest_distance = T::try_max_value()?;

            for i in 0..node.count {
                let shape_index = self.indices[node.left_child + i];
                if let Some(hit) = shapes[shape_index].intersect(ray)? {
                    if hit.distance < closest_distance {
                        closest_distance = hit.distance;
                        closest_hit = Some((shape_index, hit));
                    }
                }
            }

            return Ok(closest_hit);
        }

        // Internal node - traverse children
        let left_child_index = node.left_child;
        let right_child_index = left_child_index + 1;

        let left_hit = self.intersect_recursive(ray, shapes, left_child_index)?;
        let right_hit = self.intersect_recursive(ray, shapes, right_child_index)?;

        // Return the closest hit
        Ok(match (left_hit, right_hit) {
            (Some((left_idx, left_hit)), Some((right_idx, right_hit))) => {
                if left_hit.distance <= right_hit.distance {
                    Some((left_idx, left_hit))
                } else {
                    Some((right_idx, right_hit))
                }
            }
            (Some(hit), None) | (None, Some(hit)) => Some(hit),
            (None, None) => None,
        })
    }

    /// Recursive helper for shadow ray testing.
    fn intersect_any_recursive<B>(&self, ray: &Ray<T>, shapes: &[B], node_index: usize, max_distance: T) -> Result<bool>
    where
        B: Bounded<T> + Traceable<T>,
    {
        if node_index >= self.nodes.len() {
            return Ok(false);
        }

        let node = &self.nodes[node_index];

        // Test ray against node's bounding box
        if let Some(distance) = node.aabb.intersect_distance(ray)? {
            if distance > max_distance {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        // Leaf node - test against primitives
        if node.count > 0 {
            for i in 0..node.count {
                let shape_index = self.indices[node.left_child + i];
                if let Some(hit) = shapes[shape_index].intersect(ray)? {
                    if hit.distance <= max_distance {
                        return Ok(true);
                    }
                }
            }
            return Ok(false);
        }

        // Internal node - traverse children
        let left_child_index = node.left_child;
        let right_child_index = left_child_index + 1;

        Ok(self.intersect_any_recursive(ray, shapes, left_child_index, max_distance)?
            || self.intersect_any_recursive(ray, shapes, right_child_index, max_distance)?)
    }
}

impl<T: RealField + Copy + ToPrimitive> Bounded<T> for Bvh<T> {
    fn aabb(&self) -> Result<Cow<Aabb<T>>> {
        Ok(Cow::Borrowed(&self.nodes[0].aabb))
    }
}
