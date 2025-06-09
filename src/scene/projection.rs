use nalgebra::RealField;

/// Camera type enumeration for different projection types.
#[derive(Debug, Clone)]
pub enum Projection<T: RealField + Copy> {
    /// Perspective projection with a field of view.
    Perspective(T),
    /// Orthographic projection with a specified width.
    Orthographic(T),
}
