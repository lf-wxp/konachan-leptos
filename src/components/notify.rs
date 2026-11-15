//! Toast layer rendered on top of the application.

use leptos::prelude::*;

use crate::components::{CheckCircleIcon, InfoCircleIcon, class_list};
use crate::state::{NoticeState, NoticeTag, use_app_state};

/// Icon and colour class for `tag`.
fn icon_class(tag: NoticeTag) -> &'static str {
  match tag {
    NoticeTag::Success => "icon success",
    NoticeTag::Danger => "icon danger",
    NoticeTag::Info => "icon info",
    NoticeTag::Warning => "icon warning",
  }
}

/// Icon element for `tag`; only success gets the check mark.
fn icon(tag: NoticeTag) -> AnyView {
  let class = icon_class(tag).to_string();
  match tag {
    NoticeTag::Success => view! { <CheckCircleIcon class=class/> }.into_any(),
    NoticeTag::Danger | NoticeTag::Info | NoticeTag::Warning => {
      view! { <InfoCircleIcon class=class/> }.into_any()
    }
  }
}

/// Fixed-position stack of notifications.
#[component]
pub fn Notify() -> impl IntoView {
  let state = use_app_state();

  view! {
    <Show when=move || !state.notices.get().is_empty()>
      <div class="bk-notify" role="status" aria-live="polite">
        <For
          each=move || state.notices.get()
          key=|notice| notice.id.clone()
          children=move |notice| {
            let id = notice.id.clone();
            let class_id = notice.id.clone();
            let content = notice.content.clone();
            let tag = notice.tag;
            // `with` borrows the list: `get()` would clone every notice on
            // every toast update, once per rendered notice.
            let class = move || {
              let perishing = state.notices.with(|items| {
                items
                  .iter()
                  .any(|item| item.id == class_id && item.state == NoticeState::Perish)
              });
              class_list("notice", [perishing.then_some("perish")])
            };
            let on_animation_end = move |_| {
              let perishing = state
                .notices
                .get_untracked()
                .iter()
                .any(|item| item.id == id && item.state == NoticeState::Perish);
              if perishing {
                state.remove_notice(&id);
              }
            };
            view! {
              <div class=class on:animationend=on_animation_end>
                {icon(tag)}
                {content}
              </div>
            }
          }
        />
      </div>
    </Show>
  }
}
