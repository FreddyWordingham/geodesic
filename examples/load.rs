use chromatic::{Colour, ColourMap, LabAlpha};
use geodesic::prelude::*;
use nalgebra::{Point3, Unit};
use ndarray::Array2;
use ndarray_stats::QuantileExt;
use photo::Image;
use std::error::Error;

const COLOURS: [&str; 2] = ["#000000FF", "#FFFFFFFF"];

fn main() -> Result<(), Box<dyn Error>> {
    let assets = SerializedAssets::<f32>::load("assets.json")?.build();
    let scene = SerializedScene::<f32>::load("scene.json")?.build(&assets);

    // Camera setup
    let resolution = [600, 800]; // [height, width]
    // let camera = Camera::new(
    //     Point3::new(10.0, 10.0, 10.0), // position
    //     Point3::new(0.0, 0.0, 3.0),    // look_at
    //     90.0_f32.to_radians(),         // field_of_view
    //     resolution,
    // );
    let camera = OrthoCamera::new(
        Point3::new(0.0, 10.0, 10.0),
        Point3::new(0.0, 0.0, 3.0),
        20.0, // Much larger viewing area
        resolution,
    );

    // Render
    let sun = Point3::new(10.0, -5.0, 20.0);
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
    let mut range = max_light - min_light;
    if range.is_nan() || range == 0.0 {
        range = 1.0; // Avoid division by zero
    }
    println!("Min light: {}, Max light: {}", min_light, max_light);
    let img = light.mapv(|d| (d - min_light) / range).mapv(|d| cmap.sample(d));
    img.save("./output/image.png")?;

    Ok(())
}
