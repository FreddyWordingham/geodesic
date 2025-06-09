use nalgebra::RealField;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

use crate::{bvh::BvhConfig, geometry::Mesh, scene::Assets};

/// Serialized representation of `Assets` used by `Scene`s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAssets<T: RealField + Copy + ToPrimitive> {
    /// Bounding Volume Hierarchy configuration for applicable `Assets` and `Scene`s.
    pub bvh_config: Option<BvhConfig<T>>,
    /// List of `Mesh` files to be loaded.
    pub meshes: Vec<(String, PathBuf)>, // (identifier, file path)
}

impl<T: RealField + Copy + ToPrimitive + FromStr> SerializedAssets<T> {
    /// Construct an `Assets` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the `Mesh` files cannot be loaded.
    pub fn build(self) -> Result<Assets<T>, String> {
        let bvh_config = self.bvh_config.unwrap_or_default();
        let mut assets = Assets::empty(bvh_config.clone());
        for (name, path) in self.meshes {
            let mesh = Mesh::load(&bvh_config, path);
            assets = assets.add_mesh(&name, mesh)?;
        }
        Ok(assets)
    }
}
