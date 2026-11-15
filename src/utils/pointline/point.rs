//! A single point of the moving-lines animation.

/// A 2D point, in canvas pixels.
#[derive(Debug)]
pub struct Point {
  /// Horizontal coordinate.
  pub x: f32,
  /// Vertical coordinate.
  pub y: f32,
}

impl Point {
  /// Creates a point.
  #[must_use]
  pub fn new(x: f32, y: f32) -> Self {
    Self { x, y }
  }
}
