//! Domain models shared by the API layer and the UI.
//!
//! Everything in this module is plain data: no `web-sys`, no reactive
//! primitives. That keeps it unit-testable on the host target.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Load state of a single image download.
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImageState {
  /// The image bytes have been fully fetched.
  Loaded,
  /// Transfer is in flight (or has not started yet).
  #[default]
  Pending,
  /// The transfer failed.
  Error,
}

impl Display for ImageState {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let text = match self {
      ImageState::Loaded => "loaded",
      ImageState::Pending => "pending",
      ImageState::Error => "error",
    };
    write!(f, "{text}")
  }
}

/// A single image as returned by the backend.
#[derive(PartialEq, Clone, Deserialize, Debug, Default)]
pub struct Image {
  /// Backend identifier of the post.
  pub id: u32,
  /// Width of the sample image.
  pub sample_width: f64,
  /// Height of the sample image.
  pub sample_height: f64,
  /// URL of the sample image.
  pub sample: String,
  /// Width of the preview image.
  pub preview_width: f64,
  /// Height of the preview image.
  pub preview_height: f64,
  /// URL of the preview image.
  pub preview: String,
  /// URL of the full resolution image.
  pub url: String,
  /// Width of the full resolution image.
  pub width: f64,
  /// Height of the full resolution image.
  pub height: f64,
  /// Whether the post passed the content filter.
  pub security: bool,
  /// File name used when downloading.
  pub name: String,
  /// Raw tag string of the post.
  pub tags: Option<String>,
  /// Whether the image is already fully visible.
  pub full: Option<bool>,
}

/// Payload of the `GET /api/post` response.
#[derive(PartialEq, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageData {
  /// Images of the requested page.
  pub images: Vec<Image>,
  /// Total number of pages available for the query.
  pub count: usize,
}

/// Envelope of the `GET /api/post` response.
#[derive(PartialEq, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageRes {
  /// Status code reported by the backend.
  pub code: i32,
  /// Response payload.
  pub data: ImageData,
  /// Optional status message.
  pub msg: Option<String>,
}

/// Source format requested from the backend.
#[derive(PartialEq, Serialize, Deserialize, Default, Clone, Debug)]
pub enum Mode {
  /// JSON posts.
  #[default]
  Json,
  /// XML posts.
  Xml,
}

impl Display for Mode {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let text = match self {
      Mode::Json => "json",
      Mode::Xml => "xml",
    };
    write!(f, "{text}")
  }
}

impl Mode {
  /// Returns the opposite mode.
  #[must_use]
  pub const fn invert(&self) -> Self {
    match self {
      Mode::Json => Mode::Xml,
      Mode::Xml => Mode::Json,
    }
  }
}

/// Query parameters of a post list request.
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct FetchParams {
  page: u32,
  tags: String,
  mode: Mode,
  limit: u32,
}

impl FetchParams {
  /// Number of posts requested per page.
  pub const PAGE_LIMIT: u32 = 20;

  /// Builds the query for the given page, tag filter and mode.
  #[must_use]
  pub fn new(page: u32, tags: String, mode: Mode) -> Self {
    Self {
      page,
      tags,
      mode,
      limit: Self::PAGE_LIMIT,
    }
  }

  /// Query string pairs, in the order expected by the backend.
  #[cfg_attr(not(any(feature = "web", feature = "safe")), allow(dead_code))]
  #[must_use]
  pub fn param(&self) -> Vec<(&'static str, String)> {
    let Self {
      page,
      tags,
      mode,
      limit,
    } = self;
    vec![
      ("page", page.to_string()),
      ("tags", tags.clone()),
      ("mode", mode.to_string()),
      ("limit", limit.to_string()),
    ]
  }
}

/// Tauri command invoked for every backend call.
///
/// Only constructed by desktop builds, hence the dead-code allowance in web
/// builds.
#[cfg_attr(not(feature = "tauri"), allow(dead_code))]
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Action {
  /// Fetch a page of posts.
  GetPost,
  /// Download an image to disk.
  DownloadImage,
  /// Dismiss the desktop splash screen.
  CloseSplashscreen,
}

impl Display for Action {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let text = match self {
      Action::GetPost => "get_post",
      Action::DownloadImage => "download_image",
      Action::CloseSplashscreen => "close_splashscreen",
    };
    write!(f, "{text}")
  }
}

/// Empty Tauri command payload.
#[cfg_attr(not(feature = "tauri"), allow(dead_code))]
#[derive(Debug, Serialize, Deserialize)]
pub struct Empty;

/// Payload of the `download_image` Tauri command.
#[cfg_attr(not(feature = "tauri"), allow(dead_code))]
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadParam {
  /// Image URL to download.
  pub url: String,
}

/// Progress event emitted by the Tauri backend.
///
/// Web builds build one too, to reset a failed tile when the retry icon is
/// pressed.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct DownloadProgress {
  /// URL of the image being downloaded.
  pub url: String,
  /// Completion ratio, in the `0.0..=1.0` range.
  pub percent: f32,
  /// Current transfer state.
  pub status: ImageState,
}

/// A single download shown in the side panel.
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug, Default)]
pub struct Download {
  /// Preview URL used as the background of the progress tile.
  pub preview: String,
  /// URL of the image being downloaded.
  pub url: String,
  /// File name handed to the browser when the download is triggered.
  pub name: String,
  /// Completion percentage, in the `0.0..=100.0` range.
  pub percent: f32,
  /// Current transfer state.
  pub status: ImageState,
}
