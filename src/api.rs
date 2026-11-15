//! Transport layer: talks to the backend either over HTTP or through Tauri.

use thiserror::Error;

#[cfg(feature = "tauri")]
use crate::model::DownloadProgress;
use crate::model::{FetchParams, ImageRes};

/// Errors surfaced by the transport layer.
#[derive(Debug, Error)]
pub enum ApiError {
  /// The HTTP request failed (network, CORS or status error).
  #[cfg(any(feature = "web", feature = "safe"))]
  #[error("post request failed: {0}")]
  Request(#[from] gloo_net::Error),
  /// The response body was not valid JSON.
  #[error("could not decode the response: {0}")]
  Decode(#[from] serde_json::Error),
  /// The backend answered with a non-success HTTP status.
  #[cfg(any(feature = "web", feature = "safe"))]
  #[error("post request failed with status {0}")]
  Status(u16),
  /// The browser refused to start the download.
  #[cfg(not(feature = "tauri"))]
  #[error("download failed: {0}")]
  Download(String),
  /// The Tauri command failed.
  #[cfg(feature = "tauri")]
  #[error("tauri command failed: {0}")]
  Tauri(String),
  /// No transport feature was enabled at compile time.
  #[cfg(not(any(feature = "web", feature = "safe", feature = "tauri", feature = "fake")))]
  #[error("no transport feature enabled; enable one of `web`, `safe`, `tauri`, `fake`")]
  NoTransport,
}

/// Requests one page of posts.
///
/// # Errors
///
/// Returns [`ApiError`] when the request fails or the payload cannot be
/// decoded.
pub async fn fetch_posts(params: FetchParams) -> Result<ImageRes, ApiError> {
  #[cfg(any(feature = "web", feature = "safe"))]
  {
    let headers = gloo_net::http::Headers::new();
    // Public build marker, not a secret: it ships inside the WASM bundle and
    // must never be treated as access control by the backend.
    headers.append("x-api-key", "konachan-api");
    let response = gloo_net::http::Request::get("/api/post")
      .headers(headers)
      .query(params.param())
      .send()
      .await?;
    // `json()` does not look at the status: without this an HTML error page
    // from a proxy would surface as a decode failure.
    if !response.ok() {
      return Err(ApiError::Status(response.status()));
    }
    Ok(response.json::<ImageRes>().await?)
  }

  #[cfg(all(feature = "tauri", not(any(feature = "web", feature = "safe"))))]
  {
    Ok(
      tauri_sys::core::invoke::<ImageRes>(&crate::model::Action::GetPost.to_string(), &params)
        .await,
    )
  }

  #[cfg(all(
    feature = "fake",
    not(any(feature = "web", feature = "safe")),
    not(feature = "tauri")
  ))]
  {
    let _ = params;
    Ok(serde_json::from_str::<ImageRes>(include_str!(
      "../static/mock/post.json"
    ))?)
  }

  #[cfg(not(any(feature = "web", feature = "safe", feature = "tauri", feature = "fake")))]
  {
    let _ = params;
    Err(ApiError::NoTransport)
  }
}

/// Downloads the image at `url` under the given file name.
///
/// # Errors
///
/// Returns [`ApiError`] when the browser or the Tauri backend refuses the
/// download.
pub async fn download_image(
  url: &str,
  #[allow(unused_variables)] name: &str,
) -> Result<(), ApiError> {
  #[cfg(not(feature = "tauri"))]
  {
    let proxied = format!("/api/image?url={}", js_sys::encode_uri(url));
    crate::utils::download_file(&proxied, name)
      .map_err(|error| ApiError::Download(format!("{error:?}")))
  }

  #[cfg(feature = "tauri")]
  {
    tauri_sys::core::invoke::<()>(
      &crate::model::Action::DownloadImage.to_string(),
      &crate::model::DownloadParam {
        url: url.to_string(),
      },
    )
    .await;
    Ok(())
  }
}

/// Dismisses the desktop splash screen.
///
/// # Errors
///
/// Currently infallible; the error type is kept for symmetry with the other
/// commands.
#[cfg(feature = "tauri")]
pub async fn close_splashscreen() -> Result<(), ApiError> {
  tauri_sys::core::invoke::<()>(
    &crate::model::Action::CloseSplashscreen.to_string(),
    &crate::model::Empty,
  )
  .await;
  Ok(())
}

/// Streams download progress events emitted by the Tauri backend.
///
/// # Errors
///
/// Returns [`ApiError`] when the event channel cannot be created.
#[cfg(feature = "tauri")]
pub async fn listen_progress(mut callback: impl FnMut(DownloadProgress)) -> Result<(), ApiError> {
  use futures_util::StreamExt;

  let mut events = tauri_sys::event::listen::<DownloadProgress>("progress")
    .await
    .map_err(|error| ApiError::Tauri(format!("{error:?}")))?;
  while let Some(event) = events.next().await {
    leptos::logging::log!("progress: {:?}", event.payload);
    callback(event.payload);
  }
  Ok(())
}
