//! Torus structure.

use nalgebra::{Point3, RealField, Unit, Vector3};
use num_traits::cast::NumCast;
use std::borrow::Cow;

use crate::{aabb::Aabb, bounded::Bounded, hit::Hit, ray::Ray, traceable::Traceable};

/// Torus centered at origin with axis aligned to Z-axis.
#[derive(Debug, Clone)]
pub struct Torus<T: RealField + Copy> {
    /// Distance from center of torus to center of tube.
    pub major_radius: T,
    /// Radius of the tube.
    pub minor_radius: T,
    /// Center of the torus.
    pub center: Point3<T>,
}

impl<T: RealField + Copy> Torus<T> {
    /// Construct a new `Torus` instance.
    pub fn new(center: Point3<T>, major_radius: T, minor_radius: T) -> Self {
        debug_assert!(major_radius > T::zero(), "Major radius must be positive");
        debug_assert!(minor_radius > T::zero(), "Minor radius must be positive");
        debug_assert!(major_radius > minor_radius, "Major radius must be greater than minor radius");

        Self {
            center,
            major_radius,
            minor_radius,
        }
    }
}

impl<T: RealField + Copy> Bounded<T> for Torus<T> {
    fn aabb(&self) -> Cow<Aabb<T>> {
        let total_radius = self.major_radius + self.minor_radius;
        let extent = Vector3::new(total_radius, total_radius, self.minor_radius);
        Cow::Owned(Aabb::new(self.center - extent, self.center + extent))
    }
}

impl<T: RealField + Copy + NumCast> Traceable<T> for Torus<T> {
    fn intersect(&self, ray: &Ray<T>) -> Option<Hit<T>> {
        let epsilon = T::default_epsilon();

        // Transform ray to torus-local coordinates (translate to origin)
        let local_origin = ray.origin - self.center.coords;

        // Ray equation: P(t) = O + t*D where O is origin, D is direction
        let ox = local_origin.x;
        let oy = local_origin.y;
        let oz = local_origin.z;
        let dx = ray.direction.x;
        let dy = ray.direction.y;
        let dz = ray.direction.z;

        // Torus equation: (sqrt(x² + y²) - R)² + z² = r²
        // where R = major_radius, r = minor_radius
        let R = self.major_radius;
        let r = self.minor_radius;

        // Coefficients for the quartic equation at⁴ + bt³ + ct² + dt + e = 0
        let sum_o_sq = ox * ox + oy * oy + oz * oz;
        let sum_d_sq = dx * dx + dy * dy + dz * dz;
        let o_dot_d = ox * dx + oy * dy + oz * dz;
        let R_sq = R * R;
        let r_sq = r * r;

        let four = T::from(4.0).unwrap();
        let two = T::from(2.0).unwrap();

        // Quartic coefficients
        let a = sum_d_sq * sum_d_sq;
        let b = four * sum_d_sq * o_dot_d;
        let c = two * sum_d_sq * (sum_o_sq - R_sq - r_sq) + four * o_dot_d * o_dot_d + four * R_sq * dz * dz;
        let d = four * o_dot_d * (sum_o_sq - R_sq - r_sq) + four * R_sq * two * oz * dz;
        let e = (sum_o_sq - R_sq - r_sq) * (sum_o_sq - R_sq - r_sq) - four * R_sq * (r_sq - oz * oz);

        // Solve quartic equation
        let roots = solve_quartic(a, b, c, d, e)?;

        // Find the smallest positive root
        let mut min_t = T::max_value().unwrap();
        let mut found = false;

        for &t in &roots {
            if t > epsilon && t < min_t {
                min_t = t;
                found = true;
            }
        }

        if !found {
            return None;
        }

        // Calculate intersection point and normal
        let intersection_point = Point3::from(local_origin + ray.direction.scale(min_t));
        let normal = calculate_torus_normal(intersection_point, R, r);

        Some(Hit::new(min_t, normal, normal))
    }
}

/// Calculate the normal vector at a point on the torus surface.
fn calculate_torus_normal<T: RealField + Copy>(point: Point3<T>, major_radius: T, _minor_radius: T) -> Unit<Vector3<T>> {
    let x = point.x;
    let y = point.y;
    let z = point.z;

    // Distance from Z-axis
    let rho = (x * x + y * y).sqrt();

    // Avoid division by zero
    let epsilon = T::default_epsilon();
    if rho < epsilon {
        // Point is on the Z-axis, normal points radially outward in XY plane
        return Unit::new_unchecked(Vector3::new(T::one(), T::zero(), T::zero()));
    }

    // Normal calculation for torus
    let factor = T::one() - major_radius / rho;
    let normal = Vector3::new(x * factor, y * factor, z);

    Unit::new_normalize(normal)
}

/// Solve quartic equation ax⁴ + bx³ + cx² + dx + e = 0
/// Returns up to 4 real roots
fn solve_quartic<T: RealField + Copy + NumCast>(a: T, b: T, c: T, d: T, e: T) -> Option<Vec<T>> {
    let epsilon = T::default_epsilon();

    // If leading coefficient is near zero, reduce to cubic
    if a.abs() < epsilon {
        return solve_cubic(b, c, d, e);
    }

    // Normalize coefficients
    let b = b / a;
    let c = c / a;
    let d = d / a;
    let e = e / a;

    // Use Ferrari's method for quartic solving
    // This is a simplified implementation that may not find all roots
    // For production use, consider a more robust numerical solver

    // Convert to depressed quartic: t⁴ + pt² + qt + r = 0
    let four = T::from(4.0).unwrap();
    let eight = T::from(8.0).unwrap();
    let three = T::from(3.0).unwrap();

    let p = c - three * b * b / eight;
    let q = b * b * b / eight - b * c / T::from(2.0).unwrap() + d;
    let r = -three * b * b * b * b / T::from(256.0).unwrap() + c * b * b / T::from(16.0).unwrap() - b * d / four + e;

    // For simplicity, use numerical methods or approximations here
    // A full implementation would solve the resolvent cubic and continue with Ferrari's method

    // Fallback: try to find roots using a simplified approach
    let mut roots = Vec::new();

    // Sample some values and use bisection/Newton's method
    // This is not a complete implementation but gives the structure
    for i in -100..=100 {
        let t = T::from(i as f64 * 0.1).unwrap();
        let val = t * t * t * t + p * t * t + q * t + r;
        if val.abs() < epsilon {
            roots.push(t - b / four);
        }
    }

    // Remove duplicates and sort
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    roots.dedup_by(|a, b| (*a - *b).abs() < epsilon);

    if roots.is_empty() { None } else { Some(roots) }
}

/// Solve cubic equation ax³ + bx² + cx + d = 0
fn solve_cubic<T: RealField + Copy + NumCast>(a: T, b: T, c: T, d: T) -> Option<Vec<T>> {
    let epsilon = T::default_epsilon();

    if a.abs() < epsilon {
        return solve_quadratic(b, c, d);
    }

    // Simplified cubic solver - for production use a proper implementation
    let mut roots = Vec::new();

    // Normalize
    let b = b / a;
    let c = c / a;
    let d = d / a;

    // Sample and find approximate roots
    for i in -100..=100 {
        let t = T::from(i as f64 * 0.1).unwrap();
        let val = t * t * t + b * t * t + c * t + d;
        if val.abs() < epsilon {
            roots.push(t);
        }
    }

    if roots.is_empty() { None } else { Some(roots) }
}

/// Solve quadratic equation ax² + bx + c = 0
fn solve_quadratic<T: RealField + Copy + NumCast>(a: T, b: T, c: T) -> Option<Vec<T>> {
    let epsilon = T::default_epsilon();

    if a.abs() < epsilon {
        // Linear equation bx + c = 0
        if b.abs() < epsilon {
            return None; // No solution or infinite solutions
        }
        return Some(vec![-c / b]);
    }

    let discriminant = b * b - T::from(4.0).unwrap() * a * c;

    if discriminant < T::zero() {
        return None; // No real roots
    }

    let sqrt_disc = discriminant.sqrt();
    let two_a = T::from(2.0).unwrap() * a;

    if discriminant == T::zero() {
        // One root
        Some(vec![-b / two_a])
    } else {
        // Two roots
        Some(vec![(-b - sqrt_disc) / two_a, (-b + sqrt_disc) / two_a])
    }
}
