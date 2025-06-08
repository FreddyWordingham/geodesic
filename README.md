# Geodesic

A high-performance ray tracing library written in Rust, featuring a flexible architecture with BVH acceleration, multiple geometric primitives, and serializable scene descriptions.

## Features

- **High Performance**: BVH (Bounding Volume Hierarchy) acceleration with Surface Area Heuristic optimization
- **Flexible Geometry**: Support for spheres, planes, triangles, and complex meshes
- **Generic Design**: Works with both `f32` and `f64` floating-point precision
- **Scene Management**: Builder pattern for constructing scenes with reusable assets
- **Multiple Camera Types**: Perspective and orthographic projection support
- **Serialization**: JSON-based scene, camera, and asset descriptions
- **OBJ File Support**: Load triangle meshes from Wavefront .obj files
- **Instance System**: Efficient rendering of multiple copies with transformations
- **Parallel Processing**: Built for multi-threaded ray tracing applications

## Quick Start

### Basic Usage

```rust
use geodesic::prelude::*;

// Create a simple scene
let scene = Scene::builder()
    .add_sphere(Point3::new(0.0, 0.0, 0.0), 1.0)
    .add_triangle(
        [Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)],
        [Unit::new_normalize(Vector3::z()); 3]
    )
    .build();

// Set up camera
let camera = Camera::new(
    Point3::new(5.0, 5.0, 5.0),              // position
    Point3::new(0.0, 0.0, 0.0),              // look_at
    CameraType::Perspective(60.0_f32.to_radians()), // field of view
    [600, 800]                               // resolution [height, width]
);

// Trace a ray
let ray = camera.generate_ray([300, 400]); // pixel coordinates
if let Some((_object_index, hit)) = scene.intersect(&ray) {
    println!("Hit at distance: {}", hit.distance);
}
```

### Using Assets and Serialization

```rust
use geodesic::prelude::*;

// Load scene from JSON files
let assets = SerializedAssets::<f32>::load("assets.json")?.build();
let scene = SerializedScene::<f32>::load("scene.json")?.build(&assets);
let camera = SerializedCamera::load("camera.json")?.build();

// Render the scene
let resolution = camera.resolution();
for row in 0..resolution[0] {
    for col in 0..resolution[1] {
        let ray = camera.generate_ray([row, col]);
        if let Some((_index, hit)) = scene.intersect(&ray) {
            // Process hit...
        }
    }
}
```

## Architecture

### Core Components

- **Primitives**: `Sphere`, `Plane`, `Triangle` - Basic geometric shapes
- **Mesh**: Collections of triangles loaded from .obj files
- **Instance**: Transformed references to meshes for efficient duplication
- **Scene**: Container for all objects with BVH acceleration
- **Camera**: Ray generation for perspective and orthographic projections
- **Assets**: Resource management for meshes and configurations

### Traits

- **`Bounded<T>`**: Objects that can provide an axis-aligned bounding box
- **`Traceable<T>`**: Objects that can be intersected by rays
- **`Persistable`**: Automatic JSON serialization/deserialization

### Performance Features

- **BVH Acceleration**: Logarithmic ray-object intersection complexity
- **Surface Area Heuristic**: Optimal BVH construction for fast traversal
- **Shadow Ray Optimization**: Early termination for visibility queries
- **Parallel-Friendly**: Immutable scene data suitable for multi-threading

## File Formats

### Scene Description (`scene.json`)

```json
{
  "objects": [
    {
      "Sphere": [[0.0, 0.0, 0.0], 1.0]
    },
    {
      "Plane": [
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0]
      ]
    },
    {
      "Triangle": [
        [
          [0.0, 0.0, 0.0],
          [1.0, 0.0, 0.0],
          [0.0, 1.0, 0.0]
        ],
        [
          [0.0, 1.0, 0.0],
          [1.0, 1.0, 1.0],
          [1.0, 2.0, 1.0]
        ]
      ]
    },
    {
      "Instance": ["mesh_name", null]
    }
  ]
}
```

### Camera Configuration (`camera.json`)

```json
{
  "camera_type": {
    "Perspective": 1.047197551
  },
  "position": [10.0, 10.0, 10.0],
  "look_at": [0.0, 0.0, 3.0],
  "resolution": [1024, 1024]
}
```

### Asset Management (`assets.json`)

```json
{
  "bvh_config": {
    "traverse_cost": 1.0,
    "intersect_cost": 1.25,
    "sah_buckets": 16,
    "max_shapes_per_node": 4,
    "max_depth": 64
  },
  "meshes": [
    ["cube", "./assets/meshes/cube.obj"],
    ["tree", "./assets/meshes/tree.obj"]
  ]
}
```

## Examples

### Simple Ray Tracer

```rust
use geodesic::prelude::*;
use rayon::prelude::*;

fn render_scene() -> Result<(), Box<dyn std::error::Error>> {
    // Load scene components
    let assets = SerializedAssets::<f32>::load("assets.json")?.build();
    let scene = SerializedScene::<f32>::load("scene.json")?.build(&assets);
    let camera = SerializedCamera::load("camera.json")?.build();

    let resolution = camera.resolution();
    let sun_position = Point3::new(10.0, -5.0, 20.0);

    // Generate all pixel coordinates
    let pixels: Vec<(usize, usize)> = (0..resolution[0])
        .flat_map(|row| (0..resolution[1]).map(move |col| (row, col)))
        .collect();

    // Parallel ray tracing
    let results: Vec<f32> = pixels
        .into_par_iter()
        .map(|(row, col)| {
            let ray = camera.generate_ray([row, col]);

            if let Some((_index, hit)) = scene.intersect(&ray) {
                // Simple lighting calculation
                let hit_pos = ray.origin + ray.direction.scale(hit.distance - 0.01);
                let light_dir = Unit::new_normalize(sun_position - hit_pos);
                let diffuse = hit.geometric_normal.dot(&light_dir).max(0.0);

                // Shadow test
                let shadow_ray = Ray::new(hit_pos, light_dir);
                let in_shadow = scene.intersect_any(&shadow_ray, 100.0);

                if in_shadow { 0.1 } else { 0.1 + diffuse * 0.9 }
            } else {
                0.0 // Background
            }
        })
        .collect();

    // Save results...
    Ok(())
}
```

### BVH Configuration

```rust
use geodesic::prelude::*;

// Custom BVH settings for different use cases
let fast_build_config = BvhConfig::new(
    1.0,  // traverse_cost
    1.0,  // intersect_cost
    8,    // sah_buckets (fewer for faster build)
    8,    // max_shapes_per_node (more for faster build)
    32    // max_depth (less for faster build)
);

let quality_config = BvhConfig::new(
    1.0,  // traverse_cost
    1.5,  // intersect_cost
    32,   // sah_buckets (more for better quality)
    2,    // max_shapes_per_node (fewer for better quality)
    64    // max_depth (more for better quality)
);
```

## Dependencies

- `nalgebra`: Linear algebra operations
- `num-traits`: Numeric trait abstractions
- `serde`: Serialization framework
- `rayon`: Data parallelism (for applications)

## Performance Tips

1. **Use f32 for most applications** - Provides good precision with better performance (~2x faster than f64)
2. **Tune BVH parameters** - Adjust based on scene complexity and performance requirements
3. **Leverage parallel processing** - Scene data is immutable and thread-safe
4. **Use shadow ray optimization** - Call `intersect_any()` instead of `intersect()` for visibility tests
5. **Batch similar operations** - Process multiple rays together for better cache performance
