//! Small two-blade spinner shown while an image is loading.

use leptos::prelude::*;

/// Renders the spinner while `visible` is `true`.
#[component]
pub fn Pending(visible: Signal<bool>) -> impl IntoView {
  view! {
    <Show when=move || visible.get()>
      <section class="bk-pending">
        <div class="loader"></div>
      </section>
    </Show>
  }
}
