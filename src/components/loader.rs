//! Full-size overlay shown while a post list request is in flight.

use leptos::prelude::*;

use crate::state::use_app_state;

/// Renders the loading overlay while [`AppState::loading`] is set.
#[component]
pub fn Loader() -> impl IntoView {
  let state = use_app_state();
  view! {
    <Show when=move || state.loading.get()>
      <section class="bk-loader">
        <div class="loader"></div>
      </section>
    </Show>
  }
}
