//! Download queue shown under the search box.

use leptos::prelude::*;
use web_sys::WheelEvent;

use crate::components::Progress;
use crate::state::{downloads, use_app_state};

/// List of in-flight and finished downloads.
#[component]
pub fn DownloadList() -> impl IntoView {
  let state = use_app_state();
  let strip = NodeRef::<leptos::html::Section>::new();
  // Only the queue *length* is tracked: a progress tick rewrites an entry in
  // place, and following that would drag the strip back to the end while the
  // user is looking at an older tile.
  let count = Memo::new(move |_| state.downloads.with(Vec::len));
  // `scrollLeft` is an integer, so a high-resolution trackpad reports deltas
  // below one pixel that would round to zero and stick. The sub-pixel part is
  // carried across events until it adds up to a whole pixel.
  let remainder = RwSignal::new(0.0);

  // A new tile lands at the end, which is past the cap, so the strip follows it
  // instead of silently queueing it out of sight. A resize is deliberately not
  // followed: the strip stays wherever the user left it rather than being
  // yanked back to the end while they read an older tile.
  Effect::new(move |_| {
    let _ = count.get();
    if let Some(element) = strip.get() {
      element.set_scroll_left(element.scroll_width());
    }
  });

  view! {
    <section
      class="bk-download-list"
      node_ref=strip
      on:wheel=move |event: WheelEvent| {
        let Some(element) = strip.get_untracked() else { return };
        if element.scroll_width() - element.client_width() <= 0 {
          return;
        }
        // The strip scrolls on one axis only, and a plain mouse wheel reports
        // the vertical one: translate whichever axis moved into `scrollLeft`
        // so a wheel, a trackpad and shift+wheel all page the queue.
        let delta = if event.delta_x().abs() > event.delta_y().abs() {
          event.delta_x()
        } else {
          event.delta_y()
        };
        if delta == 0.0 {
          return;
        }
        event.prevent_default();
        remainder.update(|rest| {
          let target = *rest + delta;
          let step = target.round();
          *rest = target - step;
          if step != 0.0 {
            element.set_scroll_left(element.scroll_left() + step as i32);
          }
        });
      }
    >
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
