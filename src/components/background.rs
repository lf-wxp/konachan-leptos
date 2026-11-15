//! Random background image, which also drives the theme colours.

use leptos::prelude::*;
use web_sys::HtmlImageElement;

use crate::state::{ThemeColor, use_app_state};
use crate::utils::{bare_rgb, get_html_image_to_vec, random, target_as};

/// Number of background images shipped in `static/image`.
const BACKGROUND_COUNT: u16 = 14;

/// Colour samples to extract from the background image.
const PALETTE_SIZE: u8 = 3;

/// Full-screen background image.
#[component]
pub fn Background() -> impl IntoView {
  let state = use_app_state();
  let fallback = RwSignal::new(format!("/image/bg{}.jpg", random(0..BACKGROUND_COUNT)));
  let source = move || match state.wallpaper.get() {
    Some((_, url)) => url,
    None => fallback.get(),
  };

  let on_load = move |event: leptos::ev::Event| {
    let Some(image) = target_as::<HtmlImageElement>(&event) else {
      return;
    };
    let Ok(pixels) = get_html_image_to_vec(&image) else {
      leptos::logging::warn!("[Background] could not read the sampled pixels");
      return;
    };
    // The buffer comes from `getImageData`, which is always RGBA.
    let Ok(colors) =
      color_thief::get_palette(&pixels, color_thief::ColorFormat::Rgba, 10, PALETTE_SIZE)
    else {
      leptos::logging::warn!("[Background] could not extract a palette");
      return;
    };
    let Some(theme) = colors.first().copied() else {
      return;
    };
    let primary = colors.get(1).copied().unwrap_or(theme);
    let ancillary = colors.get(2).copied().unwrap_or(theme);

    state.theme.set(ThemeColor::new(
      bare_rgb(theme),
      bare_rgb(primary),
      bare_rgb(ancillary),
    ));

    #[cfg(feature = "tauri")]
    hide_splashscreen();
  };

  view! {
    <section class="bk-background" aria-hidden="true">
      <img src=source alt="" loading="eager" decoding="async" on:load=on_load/>
    </section>
  }
}

/// Dismisses the desktop splash screen shortly after the first paint.
#[cfg(feature = "tauri")]
fn hide_splashscreen() {
  leptos::prelude::set_timeout(
    move || {
      leptos::task::spawn_local(async {
        let _ = crate::api::close_splashscreen().await;
      });
    },
    std::time::Duration::from_millis(50),
  );
}
