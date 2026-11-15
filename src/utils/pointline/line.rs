//! Moving lines of the wallpaper animation, and their intersections.

use rand::RngExt;

use super::Point;

/// Random value in the inclusive `mn..=mx` interval.
fn random_int_from_interval(mn: f32, mx: f32) -> f32 {
  rand::rng().random_range(mn..=mx)
}

/// Intersection of two segments, when they cross within their extents.
#[must_use]
pub fn intersect_2_lines(l1: &Line, l2: &Line) -> Option<Point> {
  let p1 = &l1.a;
  let p2 = &l1.b;
  let p3 = &l2.a;
  let p4 = &l2.b;

  let denominator = (p4.y - p3.y) * (p2.x - p1.x) - (p4.x - p3.x) * (p2.y - p1.y);
  if denominator == 0.0 {
    return None;
  }
  let ua = ((p4.x - p3.x) * (p1.y - p3.y) - (p4.y - p3.y) * (p1.x - p3.x)) / denominator;
  let ub = ((p2.x - p1.x) * (p1.y - p3.y) - (p2.y - p1.y) * (p1.x - p3.x)) / denominator;

  // Both parameters must be inside the segment, not just positive: without the
  // upper bound the test reports crossings that only exist on the infinite
  // extension past an end point, drawing dots off-canvas.
  if (0.0..=1.0).contains(&ua) && (0.0..=1.0).contains(&ub) {
    Some(Point {
      x: p1.x + ua * (p2.x - p1.x),
      y: p1.y + ua * (p2.y - p1.y),
    })
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn line(ax: f32, ay: f32, bx: f32, by: f32) -> Line {
    Line {
      flag: Flag::H,
      va: 0.0,
      vb: 0.0,
      w: 0.0,
      h: 0.0,
      a: Point::new(ax, ay),
      b: Point::new(bx, by),
    }
  }

  #[test]
  fn crossing_segments_intersect() {
    let l1 = line(0.0, 0.0, 2.0, 0.0);
    let l2 = line(1.0, -1.0, 1.0, 1.0);
    let point = intersect_2_lines(&l1, &l2).expect("segments cross");
    assert_eq!((point.x, point.y), (1.0, 0.0));
  }

  #[test]
  fn parallel_segments_do_not_intersect() {
    let l1 = line(0.0, 0.0, 1.0, 0.0);
    let l2 = line(0.0, 1.0, 1.0, 1.0);
    assert!(intersect_2_lines(&l1, &l2).is_none());
  }

  #[test]
  fn crossing_beyond_a_segment_is_rejected() {
    // The infinite lines cross at (2, 2), past `l1`'s end point (1, 1).
    let l1 = line(0.0, 0.0, 1.0, 1.0);
    let l2 = line(2.0, 1.0, 2.0, 3.0);
    assert!(intersect_2_lines(&l1, &l2).is_none());
  }
}

/// Orientation of a line.
#[derive(Debug)]
pub enum Flag {
  /// Horizontal, spanning the full canvas width.
  H,
  /// Vertical, spanning the full canvas height.
  V,
}

/// A line moving along one axis and bouncing off the canvas edges.
#[derive(Debug)]
pub struct Line {
  flag: Flag,
  va: f32,
  vb: f32,
  w: f32,
  h: f32,
  /// First endpoint.
  pub a: Point,
  /// Second endpoint.
  pub b: Point,
}

impl Line {
  /// Creates a line for a `w` × `h` canvas and randomises its position and
  /// velocity.
  #[must_use]
  pub fn new(flag: Flag, w: f32, h: f32) -> Self {
    let mut line = Self {
      flag,
      va: 0.0,
      vb: 0.0,
      w,
      h,
      a: Point::new(0.0, 0.0),
      b: Point::new(0.0, 0.0),
    };
    line.init();
    line
  }

  /// Randomises endpoints and velocities.
  fn init(&mut self) {
    match self.flag {
      Flag::H => {
        self.a.x = 0.0;
        self.a.y = random_int_from_interval(0.0, self.w);
        self.b.x = self.w;
        self.b.y = random_int_from_interval(0.0, self.w);
      }
      Flag::V => {
        self.a.y = 0.0;
        self.a.x = random_int_from_interval(0.0, self.h);
        self.b.y = self.h;
        self.b.x = random_int_from_interval(0.0, self.h);
      }
    }
    self.va = random_int_from_interval(25.0, 100.0) / 100.0;
    self.vb = random_int_from_interval(25.0, 100.0) / 100.0;
  }

  /// Reverses the velocity of endpoints that left the canvas.
  fn edges(&mut self) {
    match self.flag {
      Flag::H => {
        if self.a.y < 0.0 || self.a.y > self.h {
          self.va *= -1.0;
        }
        if self.b.y < 0.0 || self.b.y > self.h {
          self.vb *= -1.0;
        }
      }
      Flag::V => {
        if self.a.x < 0.0 || self.a.x > self.w {
          self.va *= -1.0;
        }
        if self.b.x < 0.0 || self.b.x > self.w {
          self.vb *= -1.0;
        }
      }
    }
  }

  /// Advances the line by one frame.
  pub fn update(&mut self) {
    match self.flag {
      Flag::H => {
        self.a.y += self.va;
        self.b.y += self.vb;
      }
      Flag::V => {
        self.a.x += self.va;
        self.b.x += self.vb;
      }
    }
    self.edges();
  }
}
