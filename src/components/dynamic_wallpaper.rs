//! Canvas wallpaper made of moving lines and glowing intersections.

use leptos::prelude::*;

use crate::state::use_app_state;
use crate::utils::PointLine;

/// Number of lines drawn by the animation.
const LINE_COUNT: i32 = 5;

/// Animated canvas wallpaper, recoloured from the current theme.
#[component]
pub fn DynamicWallpaper() -> impl IntoView {
  let state = use_app_state();
  let canvas = NodeRef::<leptos::html::Canvas>::new();
  let point_line = PointLine::new(None, LINE_COUNT, "#ccc".to_string(), "#000".to_string());

  {
    let point_line = point_line.clone();
    Effect::new(move |_| {
      if let Some(canvas) = canvas.get() {
        let mut point_line = point_line.borrow_mut();
        point_line.set_canvas(canvas);
        point_line.init();
      }
    });
  }

  {
    let point_line = point_line.clone();
    Effect::new(move |_| {
      let (line_color, dot_color, _) = state.theme.get().colors();
      point_line.borrow_mut().set_color(line_color, dot_color);
    });
  }

  view! {
    <section class="bk-wallpaper">
      <canvas node_ref=canvas></canvas>
    </section>
  }
}
