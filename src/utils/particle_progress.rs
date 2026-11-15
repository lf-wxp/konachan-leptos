//! Particle-based download progress animation drawn on a canvas.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

use crate::utils::dom::{get_ctx, get_dpr};
use crate::utils::timer::Timer;

/// Rows of the particle grid.
const ROW: u8 = 9;
/// Columns of the particle grid.
const COLUMN: u8 = 16;
/// Number of cells in the grid.
const LEN: u8 = ROW * COLUMN;
/// Aspect ratio the preview is cropped to.
const RATIO: f64 = 16.0 / 9.0;

/// Upper-left corner of one grid cell.
#[derive(Debug)]
pub struct Section {
  /// Horizontal offset, in CSS pixels.
  pub x: f64,
  /// Vertical offset, in CSS pixels.
  pub y: f64,
}

/// Reveals a preview image cell by cell as the download progresses.
pub struct ParticleProgress {
  canvas: Option<HtmlCanvasElement>,
  ctx: Option<CanvasRenderingContext2d>,
  color: String,
  width: f64,
  height: f64,
  timer: Timer,
  section: Vec<Section>,
  image: Option<HtmlImageElement>,
  ratio: f64,
  percent: f64,
  /// Number of cells painted in full, so each frame only draws the delta.
  revealed: Cell<f64>,
  /// Whether the frame callback has been registered.
  subscribed: Cell<bool>,
  alpha: Cell<f64>,
  alpha_step: Cell<f64>,
  /// Weak, otherwise the animation owns itself and is never dropped: the
  /// timer would keep running after the tile is unmounted.
  this: Option<Weak<RefCell<Self>>>,
}

impl ParticleProgress {
  /// Creates the animation, optionally bound to a canvas and an image.
  pub fn new(
    canvas: Option<HtmlCanvasElement>,
    image: Option<HtmlImageElement>,
    color: String,
  ) -> Rc<RefCell<Self>> {
    let progress = Rc::new(RefCell::new(Self {
      canvas,
      ctx: None,
      color,
      width: 0.0,
      height: 0.0,
      image,
      section: vec![],
      ratio: 1.0,
      percent: 0.0,
      revealed: Cell::new(0.0),
      subscribed: Cell::new(false),
      alpha: Cell::new(0.0),
      alpha_step: Cell::new(0.02),
      timer: Timer::new(),
      this: None,
    }));
    progress.borrow_mut().this = Some(Rc::downgrade(&progress));
    progress
  }

  /// Scale applied when sampling the source image for a canvas of the current
  /// size.
  fn calc_ratio(&mut self) {
    if let Some(image) = &self.image {
      let natural_width = f64::from(image.natural_width());
      let natural_height = f64::from(image.natural_height());
      let width = RATIO * natural_height;
      self.ratio = if width <= natural_width {
        natural_height / self.height
      } else {
        natural_width / self.width
      };
    }
  }

  /// Builds the grid the preview is revealed through.
  fn create_section(&mut self) {
    self.section.clear();
    let size = self.width / f64::from(COLUMN);
    for row in 0..ROW {
      for column in 0..COLUMN {
        self.section.push(Section {
          x: f64::from(column) * size,
          y: f64::from(row) * size,
        });
      }
    }
  }

  /// Sets the colour of the cell being downloaded.
  pub fn set_color(&mut self, color: String) {
    self.color = color;
  }

  /// Binds the canvas, sizing it to its parent and the device pixel ratio.
  pub fn set_canvas(&mut self, canvas: HtmlCanvasElement) -> Option<()> {
    let parent = canvas.parent_element()?;
    let width = parent.client_width().unsigned_abs();
    let height = parent.client_height().unsigned_abs();
    // The backing store must use the same ratio the draw calls do: truncating
    // it (1.5 -> 1) would size the canvas smaller than the painted area, which
    // clips the preview on 125%/150% displays.
    let dpr = get_dpr().max(1.0);
    canvas.set_width((f64::from(width) * dpr).round() as u32);
    canvas.set_height((f64::from(height) * dpr).round() as u32);
    self.width = f64::from(width);
    self.height = f64::from(height);
    self.ctx = get_ctx(&canvas).ok();
    self.revealed.set(0.0);
    self.canvas = Some(canvas);
    Some(())
  }

  /// Sets the completion percentage, in the `0.0..=100.0` range.
  pub fn set_percent(&mut self, percent: f64) {
    self.percent = percent;
  }

  /// Sets the preview image.
  pub fn set_image(&mut self, image: HtmlImageElement) {
    self.image = Some(image);
  }

  /// Pulses the alpha of the in-flight cell.
  fn change_alpha(&self) {
    let mut alpha = self.alpha.get();
    alpha += self.alpha_step.get();
    if alpha > 1.0 {
      self.alpha_step.set(-0.02);
    }
    if alpha < 0.0 {
      self.alpha_step.set(0.02);
    }
    if (0.0..=1.0).contains(&alpha) {
      self.alpha.set(alpha);
    }
  }

  /// Draws the partially downloaded state.
  ///
  /// Frames are incremental: cells that were already revealed keep their
  /// pixels, so only the newly unlocked cell is drawn, and the pulsing cell is
  /// cleared before it is filled again.
  fn draw_particle(&self) -> Result<(), JsValue> {
    if self.percent >= 100.0 {
      return Ok(());
    }
    let canvas = self.canvas.as_ref().ok_or("no canvas bound")?;
    let ctx = self.ctx.as_ref().ok_or("no context bound")?;
    let image = self.image.as_ref().ok_or("no image bound")?;
    let size = self.width / f64::from(COLUMN);
    let per_section = 100.0 / f64::from(LEN);
    let current = (self.percent / per_section).ceil();
    let dpr = get_dpr();

    // A restart or a new canvas invalidates what is on screen.
    if self.revealed.get() > current {
      ctx.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
      self.revealed.set(0.0);
    }

    let revealed = self.revealed.get();
    if current > revealed {
      for (index, section) in self.section.iter().enumerate() {
        let position = index as f64;
        if position < revealed || position >= current {
          continue;
        }
        let Section { x, y } = *section;
        ctx.set_global_alpha(1.0);
        let _ = ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
          image,
          self.ratio * x,
          self.ratio * y,
          self.ratio * size,
          self.ratio * size,
          x * dpr,
          y * dpr,
          size * dpr,
          size * dpr,
        );
      }
      self.revealed.set(current);
    }

    if let Some(&Section { x, y }) = self.section.get(current as usize) {
      ctx.clear_rect(x * dpr, y * dpr, size * dpr, size * dpr);
      let alpha = self.alpha.get();
      ctx.set_fill_style_str(&self.color);
      ctx.set_global_alpha(alpha);
      self.change_alpha();
      ctx.fill_rect(x * dpr, y * dpr, size * dpr, size * dpr);
    }

    Ok(())
  }

  /// Draws the finished image and stops the animation.
  fn draw_full(&self) -> Result<(), JsValue> {
    if self.percent < 100.0 {
      return Ok(());
    }
    let canvas = self.canvas.as_ref().ok_or("no canvas bound")?;
    let ctx = self.ctx.as_ref().ok_or("no context bound")?;
    let image = self.image.as_ref().ok_or("no image bound")?;
    let dpr = get_dpr();
    ctx.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
    ctx.set_global_alpha(1.0);
    let _ = ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
      image,
      0.0,
      0.0,
      self.width * self.ratio,
      self.height * self.ratio,
      0.0,
      0.0,
      self.width * dpr,
      self.height * dpr,
    );
    self.timer.stop();
    Ok(())
  }

  /// Single frame of the animation.
  fn draw(&self) -> Result<(), JsValue> {
    self.draw_particle()?;
    self.draw_full()?;
    Ok(())
  }

  /// Registers the frame callback.
  ///
  /// The callback captures a [`Weak`] reference: a strong one would keep the
  /// animation alive through the timer, i.e. the same cycle as owning itself.
  fn subscribe(&self) {
    let Some(progress) = self.this.clone() else {
      return;
    };
    self.timer.subscribe(move || {
      let Some(progress) = progress.upgrade() else {
        return;
      };
      let _ = progress.borrow().draw();
    });
  }

  /// Computes the crop ratio, builds the grid and registers the frame
  /// callback. Must be called once the canvas and image are bound.
  ///
  /// Repeated calls (e.g. a second `load` event) rebuild the grid instead of
  /// appending to it, and never register the frame callback twice.
  pub fn init(&mut self) {
    self.calc_ratio();
    self.create_section();
    if !self.subscribed.get() {
      self.subscribe();
      self.subscribed.set(true);
    }
  }

  /// Clears the canvas and starts the frame loop, keeping the current
  /// percentage.
  ///
  /// The completion ratio is owned by the download queue, not by the
  /// animation, so it is deliberately *not* reset here. A tile whose preview
  /// finishes loading after the transfer already reported 100% (an image that
  /// is on disk already, or a very fast download) would otherwise be wiped
  /// back to zero with no further progress event to fix it, and a finished
  /// download would sit in the strip as an empty, never-completing tile.
  pub fn start(&mut self) {
    if let (Some(canvas), Some(ctx)) = (&self.canvas, &self.ctx) {
      ctx.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
    }
    self.revealed.set(0.0);
    self.timer.start();
  }

  /// Stops the animation, e.g. after a failed download.
  pub fn stop(&self) {
    self.timer.stop();
  }
}
