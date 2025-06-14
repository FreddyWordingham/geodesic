//! Bounding Volume Hierarchy configuration structure.

use nalgebra::RealField;
use serde::{Deserialize, Serialize};

use crate::{
    config::{
        DEFAULT_INTERSECT_COST, DEFAULT_MAX_DEPTH, DEFAULT_MAX_SHAPES_PER_NODE, DEFAULT_SAH_BUCKETS, DEFAULT_TRAVERSE_COST,
    },
    error::{BvhConfigError, Result},
};

/// Configuration structure for constructing a Bounding Volume Hierarchy (BVH).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `traverse_cost` is zero or negative
    /// - `intersect_cost` is zero or negative  
    /// - `sah_buckets` is zero
    /// - `max_shapes_per_node` is less than or equal to 2
    /// - `max_depth` is zero
    pub fn new(
        traverse_cost: T,
        intersect_cost: T,
        sah_buckets: usize,
        max_shapes_per_node: usize,
        max_depth: usize,
    ) -> Result<Self> {
        if traverse_cost <= T::zero() {
            return Err(BvhConfigError::InvalidTraverseCost {
                cost: format!("{traverse_cost:?}"),
            }
            .into());
        }

        if intersect_cost <= T::zero() {
            return Err(BvhConfigError::InvalidIntersectCost {
                cost: format!("{intersect_cost:?}"),
            }
            .into());
        }

        if sah_buckets == 0 {
            return Err(BvhConfigError::InvalidSahBuckets { buckets: sah_buckets }.into());
        }

        if max_shapes_per_node <= 2 {
            return Err(BvhConfigError::InvalidMaxShapesPerNode {
                count: max_shapes_per_node,
            }
            .into());
        }

        if max_depth == 0 {
            return Err(BvhConfigError::InvalidMaxDepth { depth: max_depth }.into());
        }

        Ok(Self {
            traverse_cost,
            intersect_cost,
            sah_buckets,
            max_shapes_per_node,
            max_depth,
        })
    }
}

impl<T: RealField + Copy> Default for BvhConfig<T> {
    fn default() -> Self {
        Self::new(
            T::from_f64(DEFAULT_TRAVERSE_COST).unwrap(),
            T::from_f64(DEFAULT_INTERSECT_COST).unwrap(),
            DEFAULT_SAH_BUCKETS,
            DEFAULT_MAX_SHAPES_PER_NODE,
            DEFAULT_MAX_DEPTH,
        )
        .unwrap()
    }
}
