//! Bounding Volume Hierarchy node structure with Surface Area Heuristic.

use nalgebra::RealField;
use num_traits::ToPrimitive;

use crate::{
    bvh::{Bvh, BvhConfig, BvhNode},
    geometry::Aabb,
    traits::Bounded,
};

/// Internal transient structure used for surface area heuristic (SAH) evaluation.
struct SplitCandidate<T: RealField + Copy> {
    /// Axis along which the split is evaluated. 0 = x-axis, 1 = y-axis, 2 = z-axis.
    axis: usize,
    /// Position along the axis where the split occurs.
    position: T,
    /// Cost of the split, calculated using SAH.
    cost: T,
}

/// Transient structure used in building a Bounding Volume Hierarchy (BVH).
#[derive(Debug)]
pub struct BvhBuilder<T: RealField + Copy> {
    /// Configuration parameters for the BVH.
    config: BvhConfig<T>,
    /// Indices of shapes contained in this node.
    indices: Vec<usize>,
    /// List of nodes.
    nodes: Vec<BvhNode<T>>,
    /// Current number of nodes used.
    nodes_used: usize,
}

impl<T: RealField + Copy + ToPrimitive> BvhBuilder<T> {
    /// Construct a new `BvhBuilder` instance.
    pub fn new(config: &BvhConfig<T>) -> Self {
        Self {
            config: config.clone(),
            indices: Vec::new(),
            nodes: Vec::new(),
            nodes_used: 0,
        }
    }

    /// Construct a `Bvh` from a collection of `Bounded` shapes.
    pub fn build<B: Bounded<T>>(mut self, shapes: &[B]) -> Bvh<T> {
        debug_assert!(!shapes.is_empty(), "BVH must contain at least one geometry");

        self.indices = (0..shapes.len()).collect();
        self.nodes = vec![
            BvhNode {
                aabb: Aabb::empty(),
                left_child: 0,
                count: 0,
            };
            (shapes.len() * 2) - 1
        ];

        self.nodes[0].left_child = 0;
        self.nodes[0].count = shapes.len();
        self.nodes_used = 1;

        self.update_bounds(0, shapes);
        let depth = self.subdivide(0, shapes, 0);

        self.nodes.truncate(self.nodes_used);
        self.nodes.shrink_to_fit();

        Bvh::construct_directly(self.indices, self.nodes, depth)
    }

    /// Expand the bounding box of a `BvhNode` to include all geometries contained within the node.
    fn update_bounds<B: Bounded<T>>(&mut self, index: usize, shapes: &[B]) {
        self.nodes[index].aabb = (0..self.nodes[index].count)
            .map(|i| shapes[self.indices[self.nodes[index].left_child + i]].aabb())
            .fold(self.nodes[index].aabb.clone(), |acc, aabb| acc.merge(&aabb));
    }

    /// Subdivide a `BvhNode` using Surface Area Heuristic (SAH).
    fn subdivide<B: Bounded<T>>(&mut self, index: usize, shapes: &[B], current_depth: usize) -> usize {
        // Termination criteria
        if (self.nodes[index].count <= self.config.max_shapes_per_node) || (current_depth >= self.config.max_depth) {
            return current_depth;
        }

        // Find the best split using SAH
        let best_split = match self.find_best_split(index, shapes) {
            Some(split) => split,
            None => return current_depth, // No valid split found
        };

        // Calculate cost of not splitting (leaf node cost)
        let leaf_cost = T::from_usize(self.nodes[index].count).unwrap() * self.config.intersect_cost;

        // If splitting is not beneficial, create a leaf
        if best_split.cost >= leaf_cost {
            return current_depth;
        }

        // Partition primitives based on the best split
        let mut i = self.nodes[index].left_child;
        let mut j = i + self.nodes[index].count - 1;

        while i <= j {
            let shape_index = self.indices[i];
            let centroid = shapes[shape_index].aabb().centre();

            if centroid[best_split.axis] < best_split.position {
                i += 1;
            } else {
                self.indices.swap(i, j);
                if j == 0 {
                    return current_depth;
                }
                j -= 1;
            }
        }

        let left_count = i - self.nodes[index].left_child;

        // Fallback to prevent degenerate splits
        if (left_count == 0) || (left_count == self.nodes[index].count) {
            return current_depth;
        }

        // Create child nodes
        let left_child_index = self.nodes_used;
        self.nodes_used += 1;
        let right_child_index = self.nodes_used;
        self.nodes_used += 1;

        self.nodes[left_child_index].left_child = self.nodes[index].left_child;
        self.nodes[left_child_index].count = left_count;

        self.nodes[right_child_index].left_child = i;
        self.nodes[right_child_index].count = self.nodes[index].count - left_count;

        self.nodes[index].left_child = left_child_index;
        self.nodes[index].count = 0; // Mark as internal node

        // Update bounding boxes and recursively subdivide
        self.update_bounds(left_child_index, shapes);
        self.update_bounds(right_child_index, shapes);

        let left_depth = self.subdivide(left_child_index, shapes, current_depth + 1);
        let right_depth = self.subdivide(right_child_index, shapes, current_depth + 1);

        left_depth.max(right_depth)
    }

    /// Find the best split using Surface Area Heuristic (SAH).
    fn find_best_split<B: Bounded<T>>(&self, node_index: usize, shapes: &[B]) -> Option<SplitCandidate<T>> {
        let node = &self.nodes[node_index];
        let node_surface_area = node.aabb.surface_area();

        if node_surface_area <= T::zero() {
            return None;
        }

        let mut best_split: Option<SplitCandidate<T>> = None;

        // Try all three axes
        for axis in 0..3 {
            let extent = node.aabb.maxs[axis] - node.aabb.mins[axis];
            if extent <= T::zero() {
                continue;
            }

            // Create buckets for this axis
            let mut buckets = vec![(0, Aabb::empty()); self.config.sah_buckets];

            // Assign primitives to buckets
            for i in 0..node.count {
                let shape_index = self.indices[node.left_child + i];
                let shape_aabb = shapes[shape_index].aabb();
                let centroid = shape_aabb.centre();

                let bucket_index = ((centroid[axis] - node.aabb.mins[axis]) / extent
                    * T::from_usize(self.config.sah_buckets).unwrap())
                .floor()
                .to_usize()
                .unwrap_or(0)
                .min(self.config.sah_buckets - 1);

                buckets[bucket_index].0 += 1;
                if buckets[bucket_index].0 == 1 {
                    buckets[bucket_index].1 = shape_aabb.into_owned();
                } else {
                    buckets[bucket_index].1 = buckets[bucket_index].1.merge(&shape_aabb);
                }
            }

            // Evaluate splits between buckets
            for split_bucket in 1..self.config.sah_buckets {
                // Calculate left side
                let mut left_count = 0;
                let mut left_aabb: Option<Aabb<T>> = None;
                for bucket in &buckets[..split_bucket] {
                    if bucket.0 > 0 {
                        left_count += bucket.0;
                        left_aabb = Some(
                            left_aabb
                                .as_ref()
                                .map_or_else(|| bucket.1.clone(), |aabb| aabb.merge(&bucket.1)),
                        );
                    }
                }

                // Calculate right side
                let mut right_count = 0;
                let mut right_aabb: Option<Aabb<T>> = None;
                for bucket in &buckets[split_bucket..] {
                    if bucket.0 > 0 {
                        right_count += bucket.0;
                        right_aabb = Some(
                            right_aabb
                                .as_ref()
                                .map_or_else(|| bucket.1.clone(), |aabb| aabb.merge(&bucket.1)),
                        );
                    }
                }

                // Skip invalid splits
                if left_count == 0 || right_count == 0 {
                    continue;
                }

                // Calculate SAH cost
                let left_surface_area = left_aabb.as_ref().map_or_else(T::zero, Aabb::surface_area);
                let right_surface_area = right_aabb.as_ref().map_or_else(T::zero, Aabb::surface_area);

                let cost = self.config.traverse_cost
                    + (left_surface_area / node_surface_area) * T::from_usize(left_count).unwrap() * self.config.intersect_cost
                    + (right_surface_area / node_surface_area)
                        * T::from_usize(right_count).unwrap()
                        * self.config.intersect_cost;

                let split_position = node.aabb.mins[axis]
                    + extent * T::from_usize(split_bucket).unwrap() / T::from_usize(self.config.sah_buckets).unwrap();

                if best_split.as_ref().is_none_or(|best| cost < best.cost) {
                    best_split = Some(SplitCandidate {
                        axis,
                        position: split_position,
                        cost,
                    });
                }
            }
        }

        best_split
    }
}
