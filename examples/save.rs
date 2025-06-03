use geodesic::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let assets = SerializedAssets::<f32> {
        bvh_config: Some(BvhConfig::default()),
        meshes: vec![
            ("circle".to_string(), "./assets/meshes/circle.obj".into()),
            ("cube".to_string(), "./assets/meshes/cube.obj".into()),
            ("tree".to_string(), "./assets/meshes/tree.obj".into()),
        ],
    };
    assets.save("assets.json")?;

    let objects = vec![
        SerializedSceneObject::Plane([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        SerializedSceneObject::Sphere([0.0, 0.0, 0.0], 1.0),
        SerializedSceneObject::Triangle(
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [[0.0, 1.0, 0.0], [1.0, 1.0, 1.0], [1.0, 2.0, 1.0]],
        ),
        SerializedSceneObject::Instance("tree".to_string(), None),
    ];
    let scene = SerializedScene { objects };
    println!("{}", scene.to_str().unwrap());
    scene.save("scene.json")?;
    Ok(())
}
