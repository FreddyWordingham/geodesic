use chromatic::{Colour, ColourMap, LabAlpha};
use geodesic::prelude::*;
use nalgebra::{Point3, Similarity3, Translation3, Unit, UnitQuaternion};
use ndarray::Array2;
use ndarray_stats::QuantileExt;
use photo::Image;
use std::error::Error;

const COLOURS: [&str; 2] = ["#000000FF", "#FFFFFFFF"];

fn main() -> Result<(), Box<dyn Error>> {
    // Acceleration structure configuration
    let bvh_config = BvhConfig::new(1.0, 1.25, 16, 4, 16);

    // Camera setup
    let resolution = [6000, 8000]; // [height, width]
    let camera = Camera::new(
        Point3::new(10.0, 10.0, 10.0), // position
        Point3::new(0.0, 0.0, 3.0),    // look_at
        90.0_f32.to_radians(),         // field_of_view
        resolution,
    );

    // Light sources
    let sun = Point3::new(10.0, -5.0, 20.0);

    // Meshes
    let tree_mesh = Mesh::load(&bvh_config, "./assets/meshes/tree.obj");

    // Scene setup
    let scene = Scene::builder()
        .with_bvh_config(BvhConfig::default())
        .add_sphere(Point3::new(0.0, 0.0, 0.0), 1.0)
        .add_sphere(Point3::new(0.0, 1.0, 0.0), 1.0)
        .add_instance(
            &tree_mesh,
            Similarity3::from_parts(Translation3::new(0.0, 0.0, 0.0), UnitQuaternion::identity(), 1.0).to_homogeneous(),
        )
        .build();

    // Render
    let mut light = Array2::<f32>::zeros(resolution);
    for row in 0..resolution[0] {
        if row % 50 == 0 {
            println!("Processing row {}/{}", row, resolution[0]);
        }
        for col in 0..resolution[1] {
            let ray = camera.generate_ray([row, col]);
            if let Some((_hit_instance_index, hit)) = scene.intersect(&ray) {
                // Calculate light contribution
                let ambient = 0.1;
                let hit_position = ray.origin + ray.direction.scale(hit.distance - 0.01);
                let light_dir = Unit::new_normalize(sun - hit_position);
                let diffuse = (hit.geometric_normal.dot(&light_dir)).max(0.0);

                // Check for shadows
                let shadow_ray = Ray::new(hit_position, light_dir);
                let shadow = if scene.intersect_any(&shadow_ray, 100.0) {
                    0.0 // In shadow
                } else {
                    1.0 // Not in shadow
                };

                light[[row, col]] = ambient + (diffuse * (1.0 - ambient) * shadow);
            }
        }
    }

    // Create an image from the distance data
    let cmap = ColourMap::new_uniform(&COLOURS.iter().map(|&c| LabAlpha::from_hex(c).unwrap()).collect::<Vec<_>>());
    let min_light = light.min().unwrap();
    let max_light = light.max().unwrap();
    let range = max_light - min_light;
    println!("Min light: {}, Max light: {}", min_light, max_light);
    let img = light.mapv(|d| (d - min_light) / range).mapv(|d| cmap.sample(d));
    img.save("./output/image.png")?;

    Ok(())
}
