//! Animated wallpaper: moving lines with glowing intersections.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{CanvasRenderingContext2d, Event, HtmlCanvasElement};

use crate::utils::dom::{get_ctx, get_dpr, get_window};
use crate::utils::timer::Timer;

use line::{Flag, Line, intersect_2_lines};
use point::Point;

pub mod line;
pub mod point;

/// Draws `line_num` moving lines and a dot at every intersection.
pub struct PointLine {
  canvas: Option<HtmlCanvasElement>,
  /// Cached 2D context: `get_context` is a JS call, and the draw loop runs once
  /// per frame.
  ctx: Option<CanvasRenderingContext2d>,
  line_num: i32,
  lines: Vec<Line>,
  line_color: String,
  dot_color: String,
  timer: Timer,
  /// Weak, otherwise the animation owns itself and is never dropped: that
  /// would leak the timer and the window listener for every mount.
  this: Option<Weak<RefCell<Self>>>,
  /// Window listener, kept alive for the whole animation and removed on drop.
  on_resize: Option<Closure<dyn Fn(Event)>>,
  /// Whether `init` already ran, so a second call cannot subscribe twice.
  started: bool,
}

impl PointLine {
  /// Creates the animation; call [`PointLine::init`] once the canvas is bound.
  pub fn new(
    canvas: Option<HtmlCanvasElement>,
    line_num: i32,
    line_color: String,
    dot_color: String,
  ) -> Rc<RefCell<Self>> {
    let point_line = Rc::new(RefCell::new(Self {
      canvas,
      ctx: None,
      line_num,
      lines: vec![],
      line_color,
      dot_color,
      timer: Timer::new(),
      this: None,
      on_resize: None,
      started: false,
    }));
    point_line.borrow_mut().this = Some(Rc::downgrade(&point_line));
    point_line
  }

  /// Rebuilds the line set for the current canvas size.
  fn create_lines(&mut self) {
    if let Some(canvas) = &self.canvas {
      let width = canvas.width() as f32;
      let height = canvas.height() as f32;
      self.lines = (0..self.line_num)
        .map(|index| {
          let flag = if index % 2 == 0 { Flag::H } else { Flag::V };
          Line::new(flag, width, height)
        })
        .collect();
    }
  }

  /// Resizes the canvas to its parent, re-binds the context and rebuilds the
  /// lines.
  pub fn resize(&mut self) -> Option<()> {
    // Cloned so the borrow of `self` ends before `self.ctx` is assigned.
    let canvas = self.canvas.clone()?;
    let parent = canvas.parent_element()?;
    let dpr = get_dpr().max(1.0);
    let width = (f64::from(parent.client_width()) * dpr).round();
    let height = (f64::from(parent.client_height()) * dpr).round();
    canvas.set_width(width.max(0.0) as u32);
    canvas.set_height(height.max(0.0) as u32);
    self.ctx = get_ctx(&canvas).ok();
    self.create_lines();
    Some(())
  }

  /// Updates the line and dot colours.
  pub fn set_color(&mut self, line_color: String, dot_color: String) {
    self.line_color = line_color;
    self.dot_color = dot_color;
  }

  /// Binds the canvas element.
  pub fn set_canvas(&mut self, canvas: HtmlCanvasElement) {
    // A different canvas has a different context.
    self.ctx = None;
    self.canvas = Some(canvas);
  }

  /// Draws one frame.
  fn draw(&mut self) -> Result<(), JsValue> {
    let Some(canvas) = self.canvas.as_ref() else {
      return Ok(());
    };
    let Some(ctx) = self.ctx.as_ref() else {
      return Ok(());
    };
    ctx.clear_rect(
      0.0,
      0.0,
      f64::from(canvas.width()),
      f64::from(canvas.height()),
    );

    ctx.set_stroke_style_str(&self.line_color);
    for line in &mut self.lines {
      ctx.begin_path();
      ctx.move_to(f64::from(line.a.x), f64::from(line.a.y));
      ctx.line_to(f64::from(line.b.x), f64::from(line.b.y));
      ctx.stroke();
      line.update();
    }

    ctx.set_fill_style_str(&self.dot_color);
    for (index, line) in self.lines.iter().enumerate() {
      for other in self.lines.iter().skip(index + 1) {
        let Some(point) = intersect_2_lines(line, other) else {
          continue;
        };
        ctx.begin_path();
        let _ = ctx.arc(
          f64::from(point.x),
          f64::from(point.y),
          2.0,
          0.0,
          2.0 * std::f64::consts::PI,
        );
        ctx.fill();
      }
    }
    Ok(())
  }

  /// Re-runs `resize` whenever the window size changes.
  fn bind_event(&mut self) {
    // Weak on purpose: a strong reference inside a listener that is owned by
    // `self` would keep the animation alive forever.
    let Some(point_line) = self.this.clone() else {
      return;
    };
    let closure = Closure::<dyn Fn(Event)>::new(move |_: Event| {
      if let Some(point_line) = point_line.upgrade() {
        point_line.borrow_mut().resize();
      }
    });
    get_window()
      .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
      .ok();
    // Stored instead of `forget`ed: `Drop` removes it again.
    self.on_resize = Some(closure);
  }

  /// Registers the frame callback.
  fn subscribe(&self) {
    let Some(point_line) = self.this.clone() else {
      return;
    };
    self.timer.subscribe(move || {
      let Some(point_line) = point_line.upgrade() else {
        return;
      };
      let _ = point_line.borrow_mut().draw();
    });
  }

  /// Binds the canvas, starts observing resizes and the animation loop.
  ///
  /// Idempotent: a second call is a no-op, so a re-running effect cannot end up
  /// with two listeners and two frame callbacks.
  pub fn init(&mut self) {
    if self.started {
      return;
    }
    self.started = true;
    self.resize();
    self.bind_event();
    self.subscribe();
    self.timer.start();
  }
}

impl Drop for PointLine {
  fn drop(&mut self) {
    self.timer.stop();
    if let Some(closure) = &self.on_resize {
      let _ = get_window()
        .remove_event_listener_with_callback("resize", closure.as_ref().unchecked_ref());
    }
  }
}
