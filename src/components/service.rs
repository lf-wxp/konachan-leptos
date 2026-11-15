//! Headless component owning the data fetching side effects.

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use web_sys::HtmlElement;

use crate::api::fetch_posts;
use crate::model::FetchParams;
use crate::state::{Msg, NoticeTag, PageAction, use_app_state};
use crate::utils::query_selector;

/// Sequence number of the latest post request.
///
/// Responses are applied in sequence order instead of arrival order: without
/// it a slow first request (page 1) could land after a fast one (page 2) and
/// leave the list and the pager out of sync.
static REQUEST_SEQ: AtomicU64 = AtomicU64::new(0);

/// How long a request may keep the loading overlay up, in seconds.
///
/// A promise that never settles (a hung connection, a proxy that stops
/// answering) would otherwise leave the overlay up for the rest of the session.
const REQUEST_TIMEOUT: u64 = 30;

/// Success codes the backend is known to report in `ImageRes::code`.
const SUCCESS_CODES: [i32; 2] = [0, 200];

/// Applies the theme to the `<html>` element and keeps the post list in sync
/// with the current query.
#[component]
pub fn Service() -> impl IntoView {
  let state = use_app_state();

  // Theme -> CSS custom properties on the document root, one property at a
  // time: `set_css_text` would drop any other inline style of `<html>`.
  Effect::new(move |_| {
    let declarations = state.theme.get().declarations();
    if let Some(root) = query_selector::<HtmlElement>("html") {
      let style = root.style();
      for (name, value) in declarations {
        let _ = style.set_property(name, &value);
      }
    }
  });

  // Detected language -> `<html lang>`, so assistive tech announces the page
  // in the language actually used by the notifications.
  Effect::new(move |_| {
    let tag = state.i18n.get().tag();
    if let Some(root) = query_selector::<HtmlElement>("html") {
      let _ = root.set_attribute("lang", tag);
    }
  });

  // Query -> backend. Re-runs whenever page, tags, mode or refresh change.
  // `page_total` is deliberately not read here: updating it from the response
  // would otherwise re-trigger this effect.
  Effect::new(move |_| {
    let page = state.page_current.get();
    let tags = state.tags.get();
    let mode = state.mode.get();
    let _ = state.refresh.get();

    let seq = REQUEST_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    state.loading.set(true);
    // Failsafe for a request that never settles; a newer request owns the flag
    // from here on, so only the current one may clear it.
    set_timeout(
      move || {
        if REQUEST_SEQ.load(Ordering::Relaxed) == seq && state.loading.get_untracked() {
          leptos::logging::error!("[Service] request {seq} timed out");
          state.loading.set(false);
          state.notify(
            state.i18n.get().t(Msg::GetListError),
            NoticeTag::Danger,
            Some(3),
          );
        }
      },
      Duration::from_secs(REQUEST_TIMEOUT),
    );
    spawn_local(async move {
      let result = fetch_posts(FetchParams::new(page as u32, tags, mode)).await;
      // A newer query superseded this one; its response is authoritative.
      if REQUEST_SEQ.load(Ordering::Relaxed) != seq {
        return;
      }
      state.loading.set(false);
      match result {
        Ok(response) => {
          // `code` is only trusted when the payload is empty: a non-success
          // status with data is still worth showing, and an unknown success
          // code must not turn a valid page into an error toast.
          if response.data.images.is_empty() && !SUCCESS_CODES.contains(&response.code) {
            leptos::logging::error!(
              "[Service] backend reported code {}: {}",
              response.code,
              response.msg.as_deref().unwrap_or("no message")
            );
            state.notify(
              state.i18n.get().t(Msg::GetListError),
              NoticeTag::Danger,
              Some(3),
            );
            return;
          }
          state.set_images(response.data.images);
          state.dispatch_page(PageAction::Total(response.data.count));
        }
        Err(error) => {
          leptos::logging::error!("[Service] could not load posts: {error}");
          state.notify(
            state.i18n.get().t(Msg::GetListError),
            NoticeTag::Danger,
            Some(3),
          );
        }
      }
    });
  });

  // Download progress events, only emitted by the desktop build.
  #[cfg(feature = "tauri")]
  Effect::new(move |_| {
    spawn_local(async move {
      let _ = crate::api::listen_progress(move |event| {
        state.dispatch_download(crate::state::DownloadAction::Update(event));
      })
      .await;
    });
  });
}
