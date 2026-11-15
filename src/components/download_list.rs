//! Download queue shown under the search box.

use leptos::prelude::*;

use crate::components::Progress;
use crate::state::{downloads, use_app_state};

/// List of in-flight and finished downloads.
#[component]
pub fn DownloadList() -> impl IntoView {
  let state = use_app_state();

  view! {
    <section class="bk-download-list">
      <ul class="scroll-bar">
        <For
          each=move || state.downloads.get()
          key=|item| item.url.clone()
          children=move |item| {
            let url = item.url.clone();
            // `with` borrows the queue instead of cloning it: `get()` would
            // copy every entry (two `String`s each) on every progress tick,
            // once per tile.
            let class = move || {
              format!(
                "bk-download-item {}",
                state
                  .downloads
                  .with(|items| downloads::status_of(items, &url))
              )
            };
            view! {
              <li class=class>
                <Progress image=item.preview.clone() url=item.url.clone() name=item.name.clone()/>
              </li>
            }
          }
        />
      </ul>
    </section>
  }
}
