//! Thin, panic-free wrappers around the browser APIs used by the app.

use color_thief::Color;
use rand::{RngExt, rng};
use std::ops::Range;
#[cfg(not(feature = "tauri"))]
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
#[cfg(not(feature = "tauri"))]
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
  CanvasRenderingContext2d, Document, Event, HtmlCanvasElement, HtmlImageElement, Window, window,
};
#[cfg(not(feature = "tauri"))]
use web_sys::{HtmlAnchorElement, RegistrationOptions};

/// Random integer in `range`.
pub fn random(range: Range<u16>) -> u16 {
  rng().random_range(range)
}

/// The global `window` object.
///
/// # Panics
///
/// Panics outside of a browser environment, which cannot happen for a
/// `wasm32-unknown-unknown` build.
pub fn get_window() -> Window {
  window().expect("no global `window` exists")
}

/// The global `document` object.
///
/// # Panics
///
/// Panics outside of a browser environment.
pub fn get_document() -> Document {
  get_window()
    .document()
    .expect("no global `Document` exists")
}

/// First element matching `selector`, cast to `T`.
pub fn query_selector<T: JsCast>(selector: &str) -> Option<T> {
  get_document()
    .query_selector(selector)
    .ok()
    .flatten()
    .and_then(|element| element.dyn_into::<T>().ok())
}

/// Device pixel ratio, defaulting to `1.0`.
pub fn get_dpr() -> f64 {
  window().map_or(1.0, |window| window.device_pixel_ratio())
}

/// Event target cast to `T`, when it matches.
pub fn target_as<T: JsCast>(event: &Event) -> Option<T> {
  event.target()?.dyn_into::<T>().ok()
}

/// 2D rendering context of `canvas`.
///
/// # Errors
///
/// Returns the JavaScript error when the context cannot be created.
pub fn get_ctx(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
  canvas
    .get_context("2d")?
    .ok_or_else(|| JsValue::from_str("2d context unavailable"))?
    .dyn_into::<CanvasRenderingContext2d>()
    .map_err(|_| JsValue::from_str("2d context has an unexpected type"))
}

/// Edge of the square the image is downscaled to before sampling colours.
///
/// A 3-colour palette does not need more samples, and a 64×64 buffer keeps the
/// read-back at 16 KiB instead of several megabytes for a full-screen image.
const SAMPLE_EDGE: u32 = 64;

/// Downscaled RGBA bytes of `img`, drawn into an offscreen canvas.
///
/// # Errors
///
/// Returns the JavaScript error when the canvas operations fail.
pub fn get_html_image_to_vec(img: &HtmlImageElement) -> Result<Vec<u8>, JsValue> {
  let canvas: HtmlCanvasElement = get_document().create_element("canvas")?.dyn_into()?;
  canvas.set_width(SAMPLE_EDGE);
  canvas.set_height(SAMPLE_EDGE);
  let ctx = get_ctx(&canvas)?;
  ctx.draw_image_with_html_image_element_and_dw_and_dh(
    img,
    0.0,
    0.0,
    f64::from(SAMPLE_EDGE),
    f64::from(SAMPLE_EDGE),
  )?;
  let image_data = ctx.get_image_data(0.0, 0.0, f64::from(SAMPLE_EDGE), f64::from(SAMPLE_EDGE))?;
  Ok(image_data.data().to_vec())
}

/// Formats a [`Color`] as a bare `r, g, b` triplet.
pub fn bare_rgb(color: Color) -> String {
  let Color { r, g, b } = color;
  format!("{r},{g},{b}")
}

/// Triggers a browser download of `url`.
///
/// # Errors
///
/// Returns the JavaScript error when the anchor element cannot be created or
/// inserted.
#[cfg(not(feature = "tauri"))]
pub fn download_file(url: &str, name: &str) -> Result<(), JsValue> {
  let document = get_document();
  let body = document
    .body()
    .ok_or_else(|| JsValue::from_str("missing document body"))?;
  let anchor: HtmlAnchorElement = document
    .create_element("a")?
    .dyn_into()
    .map_err(|_| JsValue::from_str("failed to create anchor element"))?;
  anchor.set_href(url);
  anchor.set_download(name);
  anchor.set_target("_blank");
  body.append_child(&anchor)?;
  anchor.click();
  body.remove_child(&anchor)?;
  Ok(())
}

/// Registers the service worker, waiting for `load` only if it has not fired.
#[cfg(not(feature = "tauri"))]
pub fn register_service_worker() {
  // The WASM bundle is loaded by a module script with a top-level `await`, so
  // `load` can already be over by the time the app mounts. Hooking the event
  // unconditionally would then register nothing at all.
  if get_document().ready_state() == "complete" {
    register_worker();
    return;
  }
  let closure = Closure::<dyn Fn(Event)>::new(move |_: Event| register_worker());
  get_window()
    .add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())
    .ok();
  closure.forget();
}

/// Registers `/sw.js` for the whole origin; failures are not surfaced because a
/// missing service worker only costs caching.
#[cfg(not(feature = "tauri"))]
fn register_worker() {
  spawn_local(async move {
    let options = RegistrationOptions::new();
    options.set_scope("/");
    let promise = get_window()
      .navigator()
      .service_worker()
      .register_with_options("/sw.js", &options);
    let _ = JsFuture::from(promise).await;
  });
}
