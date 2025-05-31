//! Bounding Volume Hierarchy configuration structure.

use nalgebra::RealField;

#[derive(Debug, Clone)]
pub struct BvhConfig<T: RealField + Copy> {
    /// Cost of intersecting a primitive.
    pub traverse_cost: T,
    /// Cost of traversing an internal node.
    pub intersect_cost: T,
    /// Number of SAH buckets to use for splitting.
    pub sah_buckets: usize,
    /// Maximum number of shapes per node before splitting.
    pub max_shapes_per_node: usize,
    /// Maximum depth of the BVH.
    pub max_depth: usize,
}

impl<T: RealField + Copy> BvhConfig<T> {
    /// Construct a new `BvhConfig` instance.
    pub fn new(traverse_cost: T, intersect_cost: T, sah_buckets: usize, max_shapes_per_node: usize, max_depth: usize) -> Self {
        assert!(traverse_cost > T::zero(), "Traverse cost must be greater than zero.");
        assert!(intersect_cost > T::zero(), "Intersect cost must be greater than zero.");
        assert!(sah_buckets > 0, "Number of SAH buckets must be greater than zero.");
        assert!(max_shapes_per_node > 3, "Maximum shapes per node must be greater than three.");
        assert!(max_depth > 0, "Maximum depth must be greater than zero.");
        Self {
            traverse_cost,
            intersect_cost,
            sah_buckets,
            max_shapes_per_node,
            max_depth,
        }
    }
}

impl Default for BvhConfig<f32> {
    fn default() -> Self {
        Self::new(1.0, 1.25, 16, 4, 64)
    }
}
