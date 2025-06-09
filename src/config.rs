/// Relative estimated cost of traversing a node in the `Bvh`.
pub const DEFAULT_TRAVERSE_COST: f64 = 1.0;
/// Relative estimated cost of performing a `Ray`-`Aabb` intersection test.
pub const DEFAULT_INTERSECT_COST: f64 = 1.25;
/// Number of SAH buckets to use for splitting.
pub const DEFAULT_SAH_BUCKETS: usize = 16;
/// Maximum number of shapes per node before splitting.
pub const DEFAULT_MAX_SHAPES_PER_NODE: usize = 4;
/// Maximum depth of the `Bvh`.
pub const DEFAULT_MAX_DEPTH: usize = 64;
