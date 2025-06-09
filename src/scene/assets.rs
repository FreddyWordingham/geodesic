//! Scene structure for `Ray` tracing.

use nalgebra::RealField;
use num_traits::ToPrimitive;
use std::collections::HashMap;

use crate::{bvh::BvhConfig, geometry::Mesh};

/// Builder for constructing `Scene` instances.
#[derive(Debug)]
pub struct Assets<T: RealField + Copy + ToPrimitive> {
    /// Bounding Volume Hierarchy configuration for applicable `Assets` constructed `Scene`s.
    pub bvh_config: BvhConfig<T>,
    /// Collection of `Mesh` instances available in `Scene`s.
    pub meshes: HashMap<String, Mesh<T>>,
}

impl<T: RealField + Copy + ToPrimitive> Assets<T> {
    /// Construct a new empty `Assets` instance.
    pub fn empty(bvh_config: BvhConfig<T>) -> Self {
        Self {
            bvh_config,
            meshes: HashMap::new(),
        }
    }

    /// Add a `Mesh` to the `Assets`.
    ///
    /// # Errors
    ///
    /// Returns an error if a `Mesh` with the same ID already exists in the `Assets`.
    pub fn add_mesh(mut self, id: &str, mesh: Mesh<T>) -> Result<Self, String> {
        if self.meshes.contains_key(id) {
            return Err(format!("Mesh with ID '{id}' already exists"));
        }
        let _prev_value = self.meshes.insert(id.into(), mesh);
        Ok(self)
    }
}
