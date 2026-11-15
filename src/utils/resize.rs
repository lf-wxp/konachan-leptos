//! Resize observation for the list container.

use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{Element, ResizeObserver, ResizeObserverEntry};

/// Keeps a `ResizeObserver` alive and writes the observed content-box width
/// into `width`.
///
/// The observer is disconnected when the handle is dropped, which happens
/// automatically when the owning component is unmounted.
pub struct WidthObserver {
  observer: ResizeObserver,
  _callback: Closure<dyn FnMut(js_sys::Array)>,
}

impl WidthObserver {
  /// Observes `node`, writing its width into `width`.
  ///
  /// Returns `None` when the node is not mounted yet or when the browser does
  /// not support `ResizeObserver`.
  pub fn new(node: NodeRef<leptos::html::Div>, width: RwSignal<f64>) -> Option<Self> {
    let callback = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
      if let Ok(entry) = entries.get(0).dyn_into::<ResizeObserverEntry>() {
        width.set(entry.content_rect().width());
      }
    });
    let observer = ResizeObserver::new(callback.as_ref().unchecked_ref()).ok()?;
    let element = node.get()?.dyn_into::<Element>().ok()?;
    observer.observe(&element);
    Some(Self {
      observer,
      _callback: callback,
    })
  }
}

impl Drop for WidthObserver {
  fn drop(&mut self) {
    self.observer.disconnect();
  }
}
