# Geodesic

[![crates.io](https://img.shields.io/crates/v/geodesic.svg)](https://crates.io/crates/geodesic)
[![Documentation](https://docs.rs/geodesic/badge.svg)](https://docs.rs/geodesic)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance ray tracing library for Rust, featuring efficient Bounding Volume Hierarchy (BVH) acceleration structures and support for a range of geometric primitives.

## Features

- 🚀 **High Performance**: Optimized BVH acceleration structures with Surface Area Heuristic (SAH)
- 📐 **Multiple Primitives**: Support for spheres, AABBs, triangles, and meshes
- 🎯 **Ray Tracing**: Efficient ray-geometry intersection testing
- 📦 **Mesh Loading**: Built-in Wavefront (.obj) file loader
- 🔄 **Instancing**: Mesh instancing with transformation matrices
- 📸 **Camera System**: Configurable perspective camera with field-of-view controls
- 🎨 **Rendering Pipeline**: Complete ray casting to distance field rendering
- 📊 **Generic Design**: Works with any real number type (f32, f64, etc.)

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
geodesic = "0.0.0"
```

### Basic Example

```rust
use geodesic::prelude::*;
use nalgebra::Point3;

// Configure BVH acceleration structure
let bvh_config = BvhConfig::new(
    1.0,    // traverse_cost
    1.25,   // intersect_cost
    16,     // sah_buckets
    4,      // max_shapes_per_node
    16      // max_depth
);

// Set up camera
let resolution = [600, 800];        // [height, width]
let camera = Camera::new(
    Point3::new(10.0, 10.0, 10.0),  // position
    Point3::new(0.0, 0.0, 3.0),     // look_at
    90.0_f32.to_radians(),          // field_of_view
    resolution,
);

// Create scene objects
let mut objects = Vec::new();
objects.push(SceneObject::Sphere(Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0)));
objects.push(SceneObject::Mesh(Mesh::load(&bvh_config, "./assets/tree.obj")));

// Build scene with BVH acceleration
let scene = Scene::new(&bvh_config, &objects);

// Render distance field
for row in 0..resolution[0] {
    for col in 0..resolution[1] {
        let ray = camera.generate_ray([row, col]);
        if let Some((_object_index, hit)) = scene.intersect(&ray) {
            let distance = hit.distance;
            // ...
        }
    }
}
```

## Core Components

### Geometry Primitives

- **Sphere**: Basic sphere primitive with center and radius
- **AABB**: Axis-Aligned Bounding Box with min/max corners
- **Triangle**: Optimised triangle with pre-computed edges and normals
- **Mesh**: Triangle mesh with BVH acceleration and .obj loading support

### Scene Management

- **Scene**: Top-level container for all objects with BVH acceleration
- **SceneObject**: Enumeration wrapper for different geometry types
- **Instance**: Mesh instancing with transformation matrices

### Acceleration Structures

- **BvhConfig**: Configurable parameters for BVH construction
- **BVH**: Bounding Volume Hierarchy with Surface Area Heuristic

### Ray Tracing

- **Ray**: Ray structure with origin, direction, and optimisation data
- **Hit**: Intersection result with distance and normal information
- **Camera**: Perspective camera for generating sampling rays

## File Format Support

### Wavefront OBJ

The library supports loading triangle meshes from Wavefront (.obj) files:

```rust
let mesh = Mesh::load(&bvh_config, "./path/to/model.obj");
```

**Supported features:**

- Vertex positions (`v`)
- Vertex normals (`vn`)
- Triangular faces (`f`)

**Requirements:**

- Meshes must be triangulated
- Faces must include both vertex and normal indices (format: `v//vn`)

## Performance Considerations

### BVH Optimization

The library uses Surface Area Heuristic (SAH) for optimal BVH construction:

- **Traverse Cost**: Higher values favor broader trees (fewer internal nodes)
- **Intersect Cost**: Higher values favor deeper trees (fewer primitives per leaf)
- **SAH Buckets**: More buckets provide better splits but trade-off with a slower construction time
- **Max Shapes Per Node**: Controls leaf size vs. tree depth trade-off

### Memory Usage

- BVH construction requires temporary memory proportional to scene size
- Mesh data is stored efficiently with pre-computed edge vectors
- Ray intersection uses stack-based traversal (no heap allocation)

## Examples

See the `simple.rs` example for a complete rendering pipeline that:

1. Sets up a scene with a sphere and mesh
2. Configures a perspective camera
3. Renders a distance field image
4. Saves the result as a PNG file
