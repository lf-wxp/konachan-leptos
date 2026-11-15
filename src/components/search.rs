//! Tag search box.

use leptos::prelude::*;
use std::time::Duration;

use crate::state::{Language, use_app_state};

/// Idle time before the typed tag is applied, in milliseconds.
const DEBOUNCE: u64 = 250;

/// Tag input; typing re-runs the query once the user pauses.
#[component]
pub fn Search() -> impl IntoView {
  let state = use_app_state();
  let value = RwSignal::new(String::new());
  // Bumped on every edit: a pending timer only applies its tag when it is
  // still the latest one, which debounces bursts without storing a handle.
  let generation = RwSignal::new(0_u32);

  // Tag clicks write `state.tags` directly. Reflect that back into the field
  // without disturbing an in-progress keystroke that already matches.
  Effect::new(move |_| {
    let tags = state.tags.get();
    if value.get_untracked() != tags {
      value.set(tags);
    }
  });

  // Debounced from `input` rather than `keypress`: `keypress` never fires for
  // paste, IME commits or a context-menu edit, which used to leave the typed
  // tag unapplied until the next keystroke.
  let on_input = move |event: leptos::ev::Event| {
    let tags = event_target_value(&event);
    value.set(tags.clone());
    generation.update(|generation| *generation = generation.wrapping_add(1));
    let ticket = generation.get_untracked();
    set_timeout(
      move || {
        if generation.get_untracked() != ticket {
          return;
        }
        // The pagination belongs to the previous tag filter: drop it so the
        // new query always starts on page 1. `Total(0)` + `Invoke(1)` cannot be
        // used here, `Invoke` rejects every page while the total is unknown.
        state.set_query(tags);
      },
      Duration::from_millis(DEBOUNCE),
    );
  };

  view! {
    <input
      class="bk-search"
      autocomplete="off"
      spellcheck="false"
      aria-label="Tags"
      placeholder=move || {
        if state.i18n.get().language() == Language::Zh {
          "标签，空格交集，-标签 排除"
        } else {
          "tags, space = AND, -tag excludes"
        }
      }
      prop:value=move || value.get()
      on:input=on_input
    />
  }
}
