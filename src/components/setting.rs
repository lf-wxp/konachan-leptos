//! Side panel toggles: content filter, post format and manual refresh.

use leptos::prelude::*;
use web_sys::KeyboardEvent;

use crate::components::class_list;
use crate::model::Mode;
use crate::state::{Language, use_app_state};
use crate::utils::is_safe;

/// Whether the bundle was compiled with Tauri support.
const IS_TAURI: bool = cfg!(feature = "tauri");

fn zh(language: Language) -> bool {
  language == Language::Zh
}

/// Whether `event` is a key that activates a custom button-like element.
fn is_activate_key(event: &KeyboardEvent) -> bool {
  matches!(event.key().as_str(), "Enter" | " ")
}

/// Toggle row of the side panel.
#[component]
pub fn Setting() -> impl IntoView {
  let state = use_app_state();
  // Safe mode is resolved at runtime so a single build can be served in both
  // regular and content-filtered mode.
  let safe_mode = is_safe();

  let security_class = move || {
    class_list(
      "bk-setting__security animation",
      [state.security.get().then_some("active")],
    )
  };
  let loading_class = move || {
    class_list(
      "bk-setting__refresh animation",
      [state.loading.get().then_some("active")],
    )
  };
  let mode_class = move || format!("bk-mode {}", state.mode.get());

  let toggle_security = move || state.security.update(|security| *security = !*security);
  let toggle_mode = move || state.mode.update(|mode| *mode = mode.invert());

  view! {
    <section class="bk-setting">
      {(!safe_mode)
        .then(|| {
          view! {
            // A `div` with switch semantics instead of a bare clickable
            // `label`: the row is reachable and operable from the keyboard.
            <article class=security_class>
              <div
                class="bk-setting__toggle"
                role="switch"
                tabindex="0"
                aria-checked=move || state.security.get().to_string()
                on:click=move |_| toggle_security()
                on:keydown=move |event: KeyboardEvent| {
                  if is_activate_key(&event) {
                    event.prevent_default();
                    toggle_security();
                  }
                }
              >
                <span class="bk-setting__fake animation"></span>
                <span class="lx-word">
                  {move || {
                    let zh = zh(state.i18n.get().language());
                    if state.security.get() {
                      if zh { "安全" } else { "Safe" }
                    } else if zh {
                      "全部"
                    } else {
                      "All"
                    }
                  }}
                </span>
              </div>
            </article>
          }
        })}

      {(IS_TAURI)
        .then(|| {
          view! {
            <article
              class=mode_class
              role="switch"
              tabindex="0"
              aria-checked=move || (state.mode.get() == Mode::Xml).to_string()
              on:click=move |_| toggle_mode()
              on:keydown=move |event: KeyboardEvent| {
                if is_activate_key(&event) {
                  event.prevent_default();
                  toggle_mode();
                }
              }
            >
              <span class="bk-mode__toggle animation"></span>
              <span class="lx-word">{move || state.mode.get().to_string()}</span>
            </article>
          }
        })}

      <article
        class=loading_class
        role="button"
        tabindex="0"
        title=move || if zh(state.i18n.get().language()) { "刷新" } else { "Refresh" }
        on:click=move |_| state.refresh.update(|refresh| *refresh = !*refresh)
        on:keydown=move |event: KeyboardEvent| {
          if is_activate_key(&event) {
            event.prevent_default();
            state.refresh.update(|refresh| *refresh = !*refresh);
          }
        }
      >
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
      </article>
    </section>
  }
}
