use nalgebra::RealField;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::{
    scene::{Assets, Scene, SceneObject},
    serialization::SerializedSceneObject,
};

/// Serialized representation of a `Scene`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedScene<T: RealField + Copy> {
    /// Objects within the `Scene`.
    pub objects: Vec<SerializedSceneObject<T>>,
}

impl<T: RealField + Copy + ToPrimitive> SerializedScene<T> {
    /// Construct a `Scene` instance.
    ///
    /// # Panics
    ///
    /// Panics if the `objects` vector is empty. A `Scene` must contain at least one object.
    pub fn build(self, assets: &Assets<T>) -> Scene<'_, T> {
        let objects: Vec<SceneObject<T>> = self.objects.into_iter().map(|obj| obj.build(assets)).collect();
        // Ensure we have at least one object to create a valid scene
        assert!(!objects.is_empty(), "Scene must contain at least one object");
        Scene::new(&assets.bvh_config, objects)
    }
}
