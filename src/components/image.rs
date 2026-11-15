//! Image tile with a pending spinner, fade-in and an error fallback.

use leptos::prelude::*;
use web_sys::HtmlImageElement;

use crate::components::{Pending, class_list};
use crate::model::ImageState;
use crate::utils::target_as;

/// Image element with load/error handling.
///
/// The frame is sized by CSS (`100%` of the tile content box), so the component
/// never needs to be re-rendered when the waterfall layout changes.
#[component]
pub fn Image(
  /// Image source URL.
  #[prop(into)]
  src: String,
  /// Fallback source used when `src` fails to load.
  #[prop(optional)]
  alternative: Option<String>,
  /// Class applied to the `<img>` element, combined with the load state.
  #[prop(into)]
  class_name: String,
  /// Alternative text; empty for decorative images.
  #[prop(default = String::new(), into)]
  description: String,
  /// Whether the image is above the fold and should not be lazy-loaded.
  #[prop(default = false)]
  priority: bool,
) -> impl IntoView {
  let state = RwSignal::new(ImageState::Pending);
  let animation_end = RwSignal::new(false);

  let image_class = move || format!("{class_name} {}", state.get());
  let frame_class = move || class_list("bk-image", [animation_end.get().then_some("animationend")]);

  let on_load = move |_| {
    if state.get_untracked() != ImageState::Error {
      state.set(ImageState::Loaded);
    }
  };
  let on_error = move |event: leptos::ev::ErrorEvent| {
    // Fall back once only. Swapping to a fallback that is itself missing would
    // otherwise re-enter this handler on every failure.
    let first_failure = state.get_untracked() != ImageState::Error;
    state.set(ImageState::Error);
    if first_failure
      && let Some(fallback) = alternative.as_deref()
      && let Some(image) = target_as::<HtmlImageElement>(&event)
    {
      image.set_src(fallback);
    }
  };
  let on_animation_end = move |_| animation_end.set(true);
  let pending = Signal::derive(move || state.get() == ImageState::Pending);

  view! {
    <div class="bk-thumb">
      <figure class=frame_class>
        <img
          src=src
          class=image_class
          alt=description
          loading=if priority { "eager" } else { "lazy" }
          decoding="async"
          on:animationend=on_animation_end
          on:error=on_error
          on:load=on_load
        />
        <Pending visible=pending/>
      </figure>
    </div>
  }
}
