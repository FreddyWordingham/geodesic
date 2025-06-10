use chromatic::{Colour, ColourMap, LabAlpha};
use geodesic::prelude::*;
use indicatif::ParallelProgressIterator;
use nalgebra::{Point3, Unit};
use ndarray::{Array2, s};
use ndarray_stats::QuantileExt;
use photo::Image;
use rayon::prelude::*;

type Precision = f32;

const COLOURS: [&str; 2] = ["#000000FF", "#FFFFFFFF"];

/// Example of loading a scene, camera, and assets from JSON files,
/// and rendering a light map using ray tracing.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assets = SerializedAssets::<Precision>::load("./inputs/assets.json")?.build()?;
    let scene = SerializedScene::<Precision>::load("./inputs/scene.json")?.build(&assets)?;
    let camera = SerializedCamera::load("./inputs/camera.json")?.build()?;

    let resolution = camera.resolution();
    let sun = Point3::new(10.0, -5.0, 20.0);
    let total_pixels = resolution[0] * resolution[1];

    // Create a vector of all pixel coordinates
    let pixel_coords: Vec<(usize, usize)> = (0..resolution[0])
        .flat_map(|row| (0..resolution[1]).map(move |col| (row, col)))
        .collect();

    // Process pixels in parallel and collect results
    let light_values: Vec<((usize, usize), Precision)> = pixel_coords
        .into_par_iter()
        .progress_count(total_pixels as u64)
        .map(|(row, col)| -> Result<((usize, usize), Precision), GeodesicError> {
            let ray = camera.generate_ray([row, col])?;
            let light_value = if let Some(hit) = scene.intersect(&ray)? {
                // Calculate light contribution
                let ambient = 0.1;
                let hit_position = ray.origin + ray.direction.scale(hit.distance - 0.01);
                let light_dir = Unit::new_normalize(sun - hit_position);
                let diffuse = (hit.geometric_normal.dot(&light_dir)).max(0.0);

                // Check for shadows
                let shadow_ray = Ray::new(hit_position, light_dir);
                let shadow = if scene.intersect_any(&shadow_ray, 100.0)? {
                    0.0 // In shadow
                } else {
                    1.0 // Not in shadow
                };

                ambient + (diffuse * (1.0 - ambient) * shadow)
            } else {
                0.0 // No hit
            };

            Ok(((row, col), light_value))
        })
        .collect::<Result<Vec<_>, GeodesicError>>()?;

    // Reconstruct the array from parallel results
    let mut light = Array2::<Precision>::zeros(*resolution);
    for ((row, col), value) in light_values {
        light[[row, col]] = value;
    }

    // Reduce the size of the light array by 2x in both dimensions
    let light = downsample_average(&light, 16);

    // Progress indication
    println!("Processed all {} pixels", total_pixels);

    // Create an image from the light data
    let cmap = ColourMap::new_uniform(&COLOURS.iter().map(|&c| LabAlpha::from_hex(c).unwrap()).collect::<Vec<_>>());
    let min_light = light.min().unwrap();
    let max_light = light.max().unwrap();
    let mut range = max_light - min_light;
    if range.is_nan() || range == 0.0 {
        range = 1.0;
    }
    println!("Min light: {}, Max light: {}", min_light, max_light);
    let img = light.mapv(|d| (d - min_light) / range).mapv(|d| cmap.sample(d));
    img.save("./output/image.png")?;

    Ok(())
}

fn downsample_average(arr: &Array2<Precision>, factor: usize) -> Array2<Precision> {
    let (rows, cols) = arr.dim();
    let new_rows = rows / factor;
    let new_cols = cols / factor;

    Array2::from_shape_fn((new_rows, new_cols), |(i, j)| {
        let window = arr.slice(s![i * factor..(i + 1) * factor, j * factor..(j + 1) * factor]);
        window.mean().unwrap_or(0.0)
    })
}
