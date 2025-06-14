# Geodesic

[![Crates.io](https://img.shields.io/crates/v/geodesic.svg)](https://crates.io/crates/geodesic)
[![Documentation](https://docs.rs/geodesic/badge.svg)](https://docs.rs/geodesic)
[![License](https://img.shields.io/crates/l/geodesic.svg)](LICENSE)

A high-performance ray tracing library for Rust, designed for both educational use and production applications. Geodesic provides a clean, type-safe API for building ray tracers with support for various geometric primitives, acceleration structures, and scene management.

## Features

- **🚀 High Performance**: Optimized ray-primitive intersection algorithms with BVH acceleration
- **🎯 Type Safety**: Generic over floating-point types with comprehensive error handling
- **📐 Rich Geometry**: Support for spheres, planes, triangles, and complex meshes
- **🏗️ Scene Management**: Flexible scene construction with asset management and instancing
- **📦 Serialization**: JSON-based scene, camera, and asset serialization
- **🎥 Camera Models**: Perspective and orthographic projections with configurable resolution
- **⚡ Acceleration Structures**: Bounding Volume Hierarchy (BVH) with Surface Area Heuristic
- **🔧 Configurable**: Extensive configuration options for performance tuning

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
geodesic = "0.1.0"
```

### Basic Ray Tracing

```rust
use geodesic::prelude::*;
use nalgebra::{Point3, Unit, Vector3};

// Create a simple scene with a sphere
let scene = Scene::builder()
    .add_sphere(Point3::new(0.0, 0.0, 0.0), 1.0)?
    .build()?;

// Set up a camera
let camera = Camera::new(
    Point3::new(0.0, 0.0, 5.0),  // position
    Point3::new(0.0, 0.0, 0.0),  // look at
    Projection::Perspective(90.0f32.to_radians()),
    [800, 600]  // resolution [height, width]
)?;

// Render a pixel
let ray = camera.generate_ray([400, 300])?;
if let Some(hit) = scene.intersect(&ray)? {
    println!("Hit at distance: {}", hit.distance);
}
```

### Loading Scenes from Files

```rust
use geodesic::prelude::*;

// Load assets (meshes, textures, etc.)
let assets = SerializedAssets::<f32>::load("assets.json")?.build()?;

// Load scene configuration
let scene = SerializedScene::<f32>::load("scene.json")?.build(&assets)?;

// Load camera settings
let camera = SerializedCamera::load("camera.json")?.build()?;

// Ready to render!
```

### Mesh Loading

```rust
use geodesic::prelude::*;

// Load a Wavefront OBJ file
let bvh_config = BvhConfig::default();
let mesh = Mesh::load(&bvh_config, "model.obj")?;

// Add to scene with transformation
let transform = Matrix4::new_translation(&Vector3::new(1.0, 0.0, 0.0));
let scene = Scene::builder()
    .add_instance(&mesh, transform)?
    .build()?;
```

## Architecture

### Core Components

- **Geometry**: Primitives like `Sphere`, `Plane`, `Triangle`, and `Mesh`
- **Ray Tracing**: `Ray` and `Hit` structures for intersection calculations
- **Acceleration**: `Bvh` (Bounding Volume Hierarchy) for fast ray-scene intersection
- **Scene Management**: `Scene`, `Camera`, and `Assets` for organizing render data
- **Serialization**: JSON-based configuration for scenes, cameras, and assets

### Traits

- **`Traceable`**: Ray intersection testing for any geometry
- **`Bounded`**: Axis-aligned bounding box computation
- **`Persistable`**: JSON serialization/deserialization

### Performance

Geodesic is designed for performance with:

- **BVH Acceleration**: O(log n) ray-scene intersection complexity
- **SIMD-Friendly**: Compatible with nalgebra's SIMD optimizations
- **Memory Efficient**: Minimal allocations during rendering
- **Parallel Ready**: Thread-safe structures for parallel rendering

## Examples

### Simple Light Map Renderer

```rust
use geodesic::prelude::*;
use rayon::prelude::*;

fn render_lightmap(
    scene: &Scene<f32>,
    camera: &Camera<f32>,
    light_pos: Point3<f32>
) -> Result<Vec<Vec<f32>>, GeodesicError> {
    let [height, width] = *camera.resolution();

    (0..height).into_par_iter().map(|row| {
        (0..width).map(|col| {
            let ray = camera.generate_ray([row, col])?;

            if let Some(hit) = scene.intersect(&ray)? {
                // Calculate lighting
                let hit_pos = ray.origin + ray.direction.scale(hit.distance);
                let light_dir = Unit::new_normalize(light_pos - hit_pos);
                let diffuse = hit.geometric_normal.dot(&light_dir).max(0.0);

                // Check shadows
                let shadow_ray = Ray::new(hit_pos, light_dir);
                let in_shadow = scene.intersect_any(&shadow_ray, 100.0)?;

                Ok(if in_shadow { 0.1 } else { 0.1 + 0.9 * diffuse })
            } else {
                Ok(0.0)
            }
        }).collect::<Result<Vec<f32>, GeodesicError>>()
    }).collect()
}
```

### Scene Configuration

Create JSON configuration files for complex scenes:

**assets.json**

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
    ["dragon", "models/dragon.obj"],
    ["bunny", "models/bunny.obj"]
  ]
}
```

**scene.json**

```json
{
  "objects": [
    {
      "Plane": [
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0]
      ]
    },
    {
      "Sphere": [[0.0, 0.0, 1.0], 1.0]
    },
    {
      "Instance": [
        "dragon",
        {
          "translation": [2.0, 0.0, 0.0],
          "rotation": [0.0, 45.0, 0.0],
          "scale": 2.0
        }
      ]
    }
  ]
}
```

**camera.json**

```json
{
  "projection": { "Perspective": 60.0 },
  "position": [5.0, 5.0, 5.0],
  "look_at": [0.0, 0.0, 1.0],
  "resolution": [1080, 1920]
}
```

## Configuration

### BVH Tuning

The BVH acceleration structure can be tuned for different scene types:

```rust
let bvh_config = BvhConfig::new(
    1.0,    // traverse_cost - cost of traversing internal nodes
    1.25,   // intersect_cost - cost of primitive intersection
    16,     // sah_buckets - buckets for Surface Area Heuristic
    4,      // max_shapes_per_node - leaf node capacity
    64      // max_depth - maximum tree depth
)?;
```

### Generic Precision

Geodesic supports different floating-point precisions:

```rust
// Single precision (faster)
type Scene32 = Scene<'static, f32>;

// Double precision (more accurate)
type Scene64 = Scene<'static, f64>;
```

## Error Handling

Geodesic provides comprehensive error handling with detailed error types:

```rust
match scene.intersect(&ray) {
    Ok(Some(hit)) => println!("Hit at {}", hit.distance),
    Ok(None) => println!("No intersection"),
    Err(GeodesicError::InvalidGeometry(msg)) => eprintln!("Geometry error: {}", msg),
    Err(GeodesicError::Math(msg)) => eprintln!("Math error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Supported File Formats

- **Wavefront OBJ**: Triangle mesh loading with vertex normals
- **JSON**: Scene, camera, and asset configuration

## Minimum Supported Rust Version (MSRV)

Geodesic requires Rust 1.70 or later.

### Development

```bash
# Clone the repository
git clone https://github.com/FreddyWordingham/geodesic.git
cd geodesic

# Create example scene and camera files
cargo run --example save

# Load and render the example scene
cargo run --example load

# Check documentation
cargo doc --open
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
