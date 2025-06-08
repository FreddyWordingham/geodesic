//! Scene structure for `Ray` tracing.

use nalgebra::RealField;
use num_traits::ToPrimitive;
use std::collections::HashMap;

use crate::prelude::*;

/// Builder for constructing `Scene` instances.
pub struct Assets<T: RealField + Copy + ToPrimitive> {
    pub bvh_config: BvhConfig<T>,
    pub meshes: HashMap<String, Mesh<T>>,
}

impl<'a, T: RealField + Copy + ToPrimitive> Assets<T> {
    /// Construct a new empty `Assets` instance.
    pub fn empty(bvh_config: BvhConfig<T>) -> Self {
        Self {
            bvh_config,
            meshes: HashMap::new(),
        }
    }

    /// Add a mesh to the assets.
    pub fn add_mesh(mut self, name: &str, mesh: Mesh<T>) -> Self {
        self.meshes.insert(name.into(), mesh);
        self
    }
}
