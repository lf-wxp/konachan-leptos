//! Particle progress tile of a single download.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::download_image;
use crate::components::RefreshIcon;
use crate::model::{DownloadProgress, ImageState};
use crate::state::{DownloadAction, NoticeTag, use_app_state};
use crate::utils::ParticleProgress;

/// Colour used before the theme has been resolved.
const DEFAULT_COLOR: &str = "red";

/// Error toast shown when a download fails.
const DOWNLOAD_ERROR: &str = "Download failed, try again!";

/// Canvas animation revealing the preview as the download progresses.
#[component]
pub fn Progress(
  /// Preview image URL.
  image: String,
  /// URL of the image being downloaded, used as the queue key.
  url: String,
  /// File name handed to the browser when the download is triggered.
  #[prop(default = String::new(), into)]
  name: String,
) -> impl IntoView {
  let state = use_app_state();
  let canvas = NodeRef::<leptos::html::Canvas>::new();
  let preview = NodeRef::<leptos::html::Img>::new();
  let progress = ParticleProgress::new(None, None, DEFAULT_COLOR.to_string());

  // One `O(1)` lookup per update: the queue is indexed by URL in `AppState`.
  let entry = {
    let url = url.clone();
    Memo::new(move |_| state.download_of(&url))
  };
  let percent = Memo::new(move |_| entry.get().map_or(0.0, |item| f64::from(item.percent)));
  let status = Memo::new(move |_| entry.get().map_or(ImageState::Pending, |item| item.status));

  {
    let progress = progress.clone();
    Effect::new(move |_| progress.borrow_mut().set_percent(percent.get()));
  }

  {
    let progress = progress.clone();
    Effect::new(move |_| {
      let (color, _, _) = state.theme.get().colors();
      progress.borrow_mut().set_color(color);
    });
  }

  {
    let progress = progress.clone();
    Effect::new(move |_| {
      if status.get() == ImageState::Error {
        progress.borrow().stop();
      }
    });
  }

  let on_load = {
    let progress = progress.clone();
    move |_| {
      let mut progress = progress.borrow_mut();
      if let Some(canvas) = canvas.get() {
        progress.set_canvas(canvas);
      }
      if let Some(preview) = preview.get() {
        progress.set_image(preview);
        progress.init();
        progress.start();
      }
    }
  };

  let retry = move |_| {
    let progress = progress.clone();
    let url = url.clone();
    let name = name.clone();
    // Nothing else clears the error state: the transfer only reports progress
    // once the backend starts sending again, so the tile would keep showing the
    // retry icon until then. Drop it back to zero first.
    state.dispatch_download(DownloadAction::Update(DownloadProgress {
      url: url.clone(),
      percent: 0.0,
      status: ImageState::Pending,
    }));
    spawn_local(async move {
      progress.borrow_mut().start();
      if download_image(&url, &name).await.is_err() {
        state.dispatch_download(DownloadAction::Update(DownloadProgress {
          url: url.clone(),
          percent: 0.0,
          status: ImageState::Error,
        }));
        state.notify(DOWNLOAD_ERROR, NoticeTag::Danger, Some(3));
      } else {
        // A browser build finishes the transfer as soon as the download is
        // handed over, so the tile has nothing left to show; the desktop build
        // keeps reporting through backend events.
        #[cfg(not(feature = "tauri"))]
        state.dispatch_download(DownloadAction::Remove(url));
      }
    });
  };

  // The retry button is always mounted so that no resource-owning closure ends
  // up inside a reactive fragment; `is-hidden` reproduces the previous
  // "only rendered on error" behaviour.
  let retry_class = move || {
    if status.get() == ImageState::Error {
      "retry"
    } else {
      "retry is-hidden"
    }
  };

  view! {
    <div class="bk-progress">
      <img src=image on:load=on_load node_ref=preview alt=""/>
      <canvas node_ref=canvas></canvas>
      <span class=retry_class on:click=retry>
        <RefreshIcon/>
      </span>
    </div>
  }
}
