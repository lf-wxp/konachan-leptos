//! Animation-frame driven timer shared by the canvas animations.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::{JsCast, closure::Closure};

use crate::utils::dom::get_window;

type Callbacks = Rc<RefCell<Vec<Rc<RefCell<dyn FnMut()>>>>>;
type FrameCallback = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

/// Runs a set of callbacks on every animation frame while started.
#[derive(Default)]
pub struct Timer {
  callbacks: Callbacks,
  running: Rc<Cell<bool>>,
  /// The frame callback itself. It is held here so that `stop` can cancel the
  /// frame still queued in the browser, and `start` can schedule it again.
  frame: FrameCallback,
  /// Handle of the queued frame, `0` when none is pending.
  handle: Rc<Cell<i32>>,
}

impl Timer {
  /// Creates a stopped timer without callbacks.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Registers a callback invoked on every frame.
  pub fn subscribe<F: 'static + FnMut()>(&self, callback: F) {
    self
      .callbacks
      .borrow_mut()
      .push(Rc::new(RefCell::new(callback)));
  }

  /// Starts the loop; calling it while running is a no-op.
  pub fn start(&self) {
    if self.running.get() {
      return;
    }
    self.running.set(true);
    self.schedule();
  }

  /// Stops the loop and cancels the frame that is still queued.
  ///
  /// Cancelling matters: a `stop` immediately followed by a `start` would
  /// otherwise leave the previously queued frame alive next to the new one,
  /// and every callback would run twice per frame from then on.
  pub fn stop(&self) {
    if !self.running.get() {
      return;
    }
    self.running.set(false);
    let handle = self.handle.replace(0);
    if handle != 0 {
      let _ = get_window().cancel_animation_frame(handle);
    }
  }

  /// Self-scheduling animation frame loop.
  fn schedule(&self) {
    let callbacks = Rc::clone(&self.callbacks);
    let running = Rc::clone(&self.running);
    // Weak on purpose: the closure is stored *inside* `frame` below, so a
    // strong clone here would make `frame` reference itself and never be
    // dropped when the timer is torn down.
    let frame = Rc::downgrade(&self.frame);
    let handle = Rc::clone(&self.handle);

    *self.frame.borrow_mut() = Some(Closure::new(move || {
      if !running.get() {
        return;
      }
      for callback in callbacks.borrow().iter() {
        callback.borrow_mut()();
      }
      if let Some(frame) = frame.upgrade()
        && let Some(closure) = frame.borrow().as_ref()
      {
        let id = get_window()
          .request_animation_frame(closure.as_ref().unchecked_ref())
          .unwrap_or(0);
        handle.set(id);
      }
    }));

    let id = match self.frame.borrow().as_ref() {
      Some(closure) => get_window()
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .unwrap_or(0),
      None => 0,
    };
    self.handle.set(id);
  }
}

impl Drop for Timer {
  fn drop(&mut self) {
    // The closure keeps a clone of `running`, so the loop must be told to stop
    // explicitly rather than relying on the reference count reaching zero.
    self.stop();
  }
}
