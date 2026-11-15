//! Pager: previous/next arrows, four page shortcuts and a jump-to-page field.

use leptos::prelude::*;
use web_sys::KeyboardEvent;

use crate::components::class_list;
use crate::state::{Msg, PageAction, use_app_state};

/// Number of page shortcuts shown around the current page.
const WINDOW_SIZE: u32 = 4;

/// Page number from which the shortcuts switch to the smaller font.
const MIDDLE_FONT_PAGE: usize = 102;

/// Whether `event` is a key that activates a button-like element.
fn is_activate_key(event: &KeyboardEvent) -> bool {
  matches!(event.key().as_str(), "Enter" | " ")
}

/// Pages displayed around `page` for a board with `pages` pages.
///
/// Mirrors the original sliding window: a block of four pages that follows the
/// current page, clamped to the first and last page.
fn page_window(page: u32, pages: u32) -> Vec<u32> {
  if pages == 0 {
    return Vec::new();
  }

  let half = WINDOW_SIZE / 2;
  let mut window: Vec<u32> = Vec::with_capacity(WINDOW_SIZE as usize);
  let push = |start: u32, window: &mut Vec<u32>| {
    for candidate in start..start.saturating_add(WINDOW_SIZE) {
      if (1..=pages).contains(&candidate) && !window.contains(&candidate) {
        window.push(candidate);
      }
    }
  };

  if page > half && page < pages.saturating_sub(half) {
    push(page - half, &mut window);
  }
  if page <= half {
    push(1, &mut window);
  }
  if page >= pages.saturating_sub(half) {
    push(pages.saturating_sub(WINDOW_SIZE - 1).max(1), &mut window);
  }

  window
}

/// Pager of the side panel.
#[component]
pub fn Nav() -> impl IntoView {
  let state = use_app_state();
  let pages = Memo::new(move |_| {
    page_window(
      state.page_current.get() as u32,
      state.page_total.get() as u32,
    )
  });
  let goto_value = RwSignal::new(String::new());

  let prev_class = move || {
    let disabled = state.page_current.get().saturating_sub(1) == 0;
    class_list("bk-pager_nav", [disabled.then_some("disabled")])
  };
  let next_class = move || {
    let disabled = state
      .page_total
      .get()
      .saturating_sub(state.page_current.get())
      == 0;
    class_list("bk-pager_nav", [disabled.then_some("disabled")])
  };
  let root_class = move || class_list("bk-pager", [(!pages.get().is_empty()).then_some("active")]);

  // Parsed on activation, not on every keystroke: parsing eagerly turned any
  // non-numeric edit into `0` and wrote it straight back into the field, so the
  // input could neither be cleared nor retyped.
  let goto = move || {
    if let Ok(page) = goto_value.get_untracked().trim().parse::<usize>() {
      state.dispatch_page(PageAction::Invoke(page));
    }
  };

  view! {
    <section class="bk-nav">
      <div class=root_class>
        <span
          class=prev_class
          role="button"
          tabindex="0"
          aria-label=move || state.i18n.get().t(Msg::PreviousPage)
          on:click=move |_| state.dispatch_page(PageAction::Prev)
          on:keydown=move |event: KeyboardEvent| {
            if is_activate_key(&event) {
              event.prevent_default();
              state.dispatch_page(PageAction::Prev);
            }
          }
        >
          <i></i>
        </span>
        <span
          class=next_class
          role="button"
          tabindex="0"
          aria-label=move || state.i18n.get().t(Msg::NextPage)
          on:click=move |_| state.dispatch_page(PageAction::Next)
          on:keydown=move |event: KeyboardEvent| {
            if is_activate_key(&event) {
              event.prevent_default();
              state.dispatch_page(PageAction::Next);
            }
          }
        >
          <i></i>
        </span>
        <div class="bk-pager_con">
          <ul class="bk-pager_box">
            <For
              each=move || pages.get()
              key=|page| *page
              children=move |page| {
                let item_class = move || {
                  let current = state.page_current.get();
                  class_list(
                    "bk-pager_item",
                    [
                      (current == page as usize).then_some("current"),
                      (current >= MIDDLE_FONT_PAGE).then_some("middle"),
                    ],
                  )
                };
                view! {
                  <li
                    class=item_class
                    data-id=page.to_string()
                    role="button"
                    tabindex="0"
                    aria-label=format!("{} {page}", state.i18n.get().t(Msg::Page))
                    on:click=move |_| state.dispatch_page(PageAction::Invoke(page as usize))
                    on:keydown=move |event: KeyboardEvent| {
                      if is_activate_key(&event) {
                        event.prevent_default();
                        state.dispatch_page(PageAction::Invoke(page as usize));
                      }
                    }
                  >
                    <span class="bk-pager_item-text">{page}</span>
                  </li>
                }
              }
            />
          </ul>
        </div>
        <div class="bk-pager_go">
          <em class="bk-pager_go-em"></em>
          <div class="bk-pager_go-div">
            <span class="bk-pager_go-span">{move || state.page_total.get()}</span>
          </div>
          <div class="bk-pager_go-div">
            <input
              class="bk-pager_go-input animation"
              type="text"
              placeholder="page"
              prop:value=move || goto_value.get()
              on:input=move |event| goto_value.set(event_target_value(&event))
              on:keydown=move |event: KeyboardEvent| {
                if event.key() == "Enter" {
                  event.prevent_default();
                  goto();
                }
              }
            />
          </div>
          <button class="bk-pager_btn" on:click=move |_| goto()>
            <span></span>
          </button>
        </div>
        <div class="bk-pager_holder"></div>
      </div>
    </section>
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn window_is_empty_without_pages() {
    assert!(page_window(1, 0).is_empty());
  }

  #[test]
  fn window_starts_at_the_first_page() {
    assert_eq!(page_window(1, 10), vec![1, 2, 3, 4]);
    assert_eq!(page_window(2, 10), vec![1, 2, 3, 4]);
  }

  #[test]
  fn window_follows_the_current_page() {
    assert_eq!(page_window(5, 10), vec![3, 4, 5, 6]);
  }

  #[test]
  fn window_ends_at_the_last_page() {
    assert_eq!(page_window(9, 10), vec![7, 8, 9, 10]);
    assert_eq!(page_window(10, 10), vec![7, 8, 9, 10]);
  }

  #[test]
  fn window_never_contains_duplicates_or_out_of_range_pages() {
    for pages in 1..=8u32 {
      for page in 1..=pages {
        let window = page_window(page, pages);
        assert!(
          window
            .iter()
            .all(|candidate| (1..=pages).contains(candidate)),
          "page {page}/{pages} produced {window:?}"
        );
        let mut unique = window.clone();
        unique.dedup();
        assert_eq!(
          unique.len(),
          window.len(),
          "page {page}/{pages} repeated a page"
        );
        assert!(
          window.contains(&page),
          "page {page}/{pages} is not reachable"
        );
      }
    }
  }
}
