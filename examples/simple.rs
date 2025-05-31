use chromatic::{Colour, ColourMap, HslAlpha};
use nalgebra::{Matrix4, Point3, Unit, Vector3};
use ndarray::Array2;
use photo::Image;
use ptolemy::prelude::*;
use std::path::Path;

const COLOURS: [&str; 4] = ["#000000FF", "#FF0000FF", "#00FF00FF", "#FFFFFFFF"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting up Ptolemy ray tracing scene...");

    // Create BVH configuration for optimal performance
    let bvh_config = BvhConfig::new(
        1.0,  // traverse_cost
        1.25, // intersect_cost
        16,   // sah_buckets
        4,    // max_shapes_per_node
        16,   // max_depth
    );

    // Create scene objects vector
    let mut objects = Vec::new();

    // 1. Add a Sphere
    let sphere = Sphere::new(
        Point3::new(-2.0, 0.0, 0.0), // center
        1.0,                         // radius
    );
    objects.push(SceneObject::Sphere(sphere));
    println!("Added sphere at (-2, 0, 0) with radius 1.0");

    // 2. Add a Triangle
    let triangle_vertices = [
        Point3::new(2.0, -1.0, -1.0),
        Point3::new(3.0, -1.0, -1.0),
        Point3::new(2.5, 1.0, -1.0),
    ];
    let triangle_normals = [
        Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0)),
        Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0)),
        Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0)),
    ];
    let triangle = Triangle::new(triangle_vertices, triangle_normals);
    objects.push(SceneObject::Triangle(triangle));
    println!("Added triangle at (2-3, -1 to 1, -1)");

    // // 3. Load and add a Mesh (teapot)
    // let teapot_path = Path::new("./assets/meshes/teapot.obj");
    // if teapot_path.exists() {
    //     let teapot_mesh = Mesh::load(bvh_config.clone(), teapot_path);
    //     objects.push(SceneObject::Mesh(teapot_mesh));
    //     println!("Loaded teapot mesh from ./assets/meshes/teapot.obj");
    // } else {
    //     println!("Warning: teapot.obj not found, creating a simple mesh instead");
    //     // Create a simple pyramid mesh as fallback
    //     let pyramid_triangles = create_pyramid_mesh();
    //     let pyramid_mesh = Mesh::new(bvh_config.clone(), pyramid_triangles);
    //     objects.push(SceneObject::Mesh(pyramid_mesh));
    //     println!("Created fallback pyramid mesh");
    // }

    // 4. Load and add an Instance (tree)
    let tree_path = Path::new("./assets/meshes/tree.obj");
    // Load the tree mesh
    let tree_mesh = Mesh::load(bvh_config.clone(), tree_path);

    // Create a transformation matrix (translate and scale)
    let transform = Matrix4::new_translation(&Vector3::new(0.0, 0.0, 0.0)) * Matrix4::new_scaling(0.5);

    // Create an instance with the transformation
    // let tree_instance = Instance::new(&tree_mesh, transform);
    objects.push(SceneObject::Mesh(tree_mesh));

    // Note: We need to handle the lifetime properly
    // In a real application, you'd store the mesh separately
    println!("Loaded tree mesh instance from ./assets/meshes/tree.obj");
    println!("Warning: Instance requires careful lifetime management in this example");

    // Create the scene with BVH acceleration
    if !objects.is_empty() {
        let scene = Scene::new(bvh_config.clone(), &objects);
        println!("Created scene with {} objects", objects.len());

        // Set up camera
        let resolution = [480, 640]; // [height, width]
        let camera = Camera::new(
            Point3::new(10.0, 12.0, 15.0), // position
            Point3::new(0.0, 0.0, 0.0),    // look_at
            std::f32::consts::PI / 4.0,    // field_of_view (45 degrees)
            resolution,                    // resolution
        );
        println!("Created camera at (0, 2, 5) looking at origin");

        // Demonstrate ray generation and intersection testing
        println!("\nTesting ray intersections...");

        // Test a few sample rays
        let test_pixels = [
            [240, 320], // Center pixel
            [120, 160], // Top-left quadrant
            [360, 480], // Bottom-right quadrant
        ];

        for &pixel in &test_pixels {
            let ray = camera.generate_ray(pixel);

            if let Some((object_index, hit)) = scene.intersect(&ray) {
                println!(
                    "Pixel {:?}: Hit object {} at distance {:.3}",
                    pixel, object_index, hit.distance
                );
                println!(
                    "  Normal: ({:.3}, {:.3}, {:.3})",
                    hit.geometric_normal.x, hit.geometric_normal.y, hit.geometric_normal.z
                );
            } else {
                println!("Pixel {:?}: No intersection", pixel);
            }
        }

        // Demonstrate shadow ray testing
        println!("\nTesting shadow rays...");
        let light_pos = Point3::new(2.0, 3.0, 2.0);
        let surface_point = Point3::new(0.0, 0.0, 0.0);
        let shadow_direction = Unit::new_normalize(light_pos - surface_point);
        let shadow_ray = Ray::new(surface_point, shadow_direction);
        let max_distance = (light_pos - surface_point).magnitude();

        if scene.intersect_any(&shadow_ray, max_distance) {
            println!("Shadow ray blocked - point is in shadow");
        } else {
            println!("Shadow ray clear - point is illuminated");
        }

        println!("\nRay tracing setup complete!");
        println!("Scene contains:");
        println!("- {} total objects", objects.len());
        println!("- BVH acceleration structure");
        // println!("- Camera with {}x{} resolution", camera.resolution[1], camera.resolution[0]);

        let cmap = ColourMap::new_uniform(
            COLOURS
                .iter()
                .map(|&c| HslAlpha::<f32>::from_hex(c).unwrap())
                .collect::<Vec<_>>()
                .as_slice(),
        );
        let mut distance = Array2::from_elem(resolution, 0.0);

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

        // Normalize distances but also encode instance information
        let (min_dist, max_dist) = distance
            .iter()
            .filter(|&&x| x > 0.0) // Only consider hits
            .fold((std::f32::MAX, 0.0), |(min, max): (f32, f32), &val| {
                (min.min(val), max.max(val))
            });
        println!("Min distance: {}, Max distance: {}", min_dist, max_dist);

        let img = distance.mapv(|d| {
            if d > 0.0 {
                let normalized = (d - min_dist) / (max_dist - min_dist);
                cmap.sample(normalized)
            } else {
                HslAlpha::from_hex("#000000FF").unwrap() // Background color for no hits
            }
        });

        img.save(Path::new("./output/scene_image.png")).unwrap();
        println!("Image saved to ./output/scene_image.png");
    } else {
        println!("No objects were added to the scene!");
    }

    Ok(())
}

// Helper function to create a simple pyramid mesh as fallback
fn create_pyramid_mesh() -> Vec<Triangle<f32>> {
    let vertices = [
        Point3::new(0.0, 1.0, 0.0),    // apex
        Point3::new(-1.0, -1.0, -1.0), // base corner 1
        Point3::new(1.0, -1.0, -1.0),  // base corner 2
        Point3::new(1.0, -1.0, 1.0),   // base corner 3
        Point3::new(-1.0, -1.0, 1.0),  // base corner 4
    ];

    let mut triangles = Vec::new();

    // Create triangular faces of the pyramid
    let faces = [
        // Side faces
        ([0, 1, 2], Vector3::new(-0.5, 0.5, -1.0).normalize()),
        ([0, 2, 3], Vector3::new(1.0, 0.5, -0.5).normalize()),
        ([0, 3, 4], Vector3::new(0.5, 0.5, 1.0).normalize()),
        ([0, 4, 1], Vector3::new(-1.0, 0.5, 0.5).normalize()),
        // Base faces
        ([1, 4, 3], Vector3::new(0.0, -1.0, 0.0)),
        ([1, 3, 2], Vector3::new(0.0, -1.0, 0.0)),
    ];

    for (face_vertices, normal) in faces.iter() {
        let triangle_vertices = [
            vertices[face_vertices[0]],
            vertices[face_vertices[1]],
            vertices[face_vertices[2]],
        ];
        let unit_normal = Unit::new_normalize(*normal);
        let triangle_normals = [unit_normal, unit_normal, unit_normal];

        triangles.push(Triangle::new(triangle_vertices, triangle_normals));
    }

    triangles
}
