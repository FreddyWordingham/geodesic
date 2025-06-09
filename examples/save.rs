use geodesic::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Camera
    let camera = SerializedCamera {
        projection: SerializedProjection::Perspective(90.0_f32.to_radians()), // camera projection mode
        position: [10.0, 10.0, 10.0],                                         // view point
        look_at: [0.0, 0.0, 3.0],                                             // target point
        resolution: [600, 800],                                               // [height, width]
    };
    camera.save("./inputs/camera.json")?;

    // Assets
    let assets = SerializedAssets::<f32> {
        bvh_config: Some(BvhConfig::default()),
        meshes: vec![
            ("circle".to_string(), "./assets/meshes/circle.obj".into()),
            ("cube".to_string(), "./assets/meshes/cube.obj".into()),
            ("tree".to_string(), "./assets/meshes/tree.obj".into()),
        ],
    };
    assets.save("./inputs/assets.json")?;

    // Scene
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
    scene.save("./inputs/scene.json")?;
    Ok(())
}
