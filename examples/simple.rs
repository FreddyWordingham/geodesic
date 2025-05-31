use chromatic::{Colour, ColourMap, LabAlpha};
use geodesic::prelude::*;
use nalgebra::Point3;
use ndarray::Array2;
use ndarray_stats::QuantileExt;
use photo::Image;
use std::error::Error;

const COLOURS: [&str; 2] = ["#000000FF", "#FFFFFFFF"];

fn main() -> Result<(), Box<dyn Error>> {
    // Acceleration structure configuration
    let bvh_config = BvhConfig::new(1.0, 1.25, 16, 4, 16);

    // Camera setup
    let resolution = [600, 800]; // [height, width]
    let camera = Camera::new(
        Point3::new(10.0, 10.0, 10.0), // position
        Point3::new(0.0, 0.0, 3.0),    // look_at
        90.0_f32.to_radians(),         // field_of_view
        resolution,
    );

    // Scene setup
    let mut objects = Vec::new();
    objects.push(SceneObject::Sphere(Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0)));
    objects.push(SceneObject::Mesh(Mesh::load(&bvh_config, "./assets/meshes/tree.obj")));
    let scene = Scene::new(&bvh_config, &objects);

    // Render
    let mut distance = Array2::<f32>::zeros(resolution);
    for row in 0..resolution[0] {
        if row % 50 == 0 {
            println!("Processing row {}/{}", row, resolution[0]);
        }
        for col in 0..resolution[1] {
            let ray = camera.generate_ray([row, col]);
            if let Some((_hit_instance_index, hit)) = scene.intersect(&ray) {
                distance[[row, col]] = hit.distance;
            }
        }
    }

    // Create an image from the distance data
    let cmap = ColourMap::new_uniform(&COLOURS.iter().map(|&c| LabAlpha::from_hex(c).unwrap()).collect::<Vec<_>>());
    let min_dist = distance.min().unwrap();
    let max_dist = distance.max().unwrap();
    let range = max_dist - min_dist;
    println!("Min distance: {}, Max distance: {}", min_dist, max_dist);
    let img = distance.mapv(|d| (d - min_dist) / range).mapv(|d| cmap.sample(d));
    img.save("./output/image.png")?;

    Ok(())
}
