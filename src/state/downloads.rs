//! Download queue and its pure reducer.

use crate::model::{Download, DownloadProgress, ImageState};

/// Actions that can be applied to the download queue.
///
/// Web builds dispatch [`DownloadAction::Update`] from the retry button; the
/// rest of the queue is driven by the desktop backend.
#[derive(Clone, Debug, PartialEq)]
pub enum DownloadAction {
  /// Append a download to the end of the queue.
  ///
  /// Only the ordered variants are used by the UI today; this one keeps the
  /// reducer symmetric and is exercised by the unit tests.
  #[allow(dead_code)]
  Push(Download),
  /// Insert a download at the front of the queue.
  ///
  /// Used by every transport: the desktop backend pushes its own events, while
  /// browser builds queue the entry themselves so the download button can be
  /// disabled and a failed transfer can be retried from the strip.
  UnShift(Download),
  /// Update the progress of the download matching `progress.url`.
  Update(DownloadProgress),
  /// Drop the download matching `url` from the queue.
  ///
  /// Emitted once a browser-build download has been handed to the browser (it
  /// completes immediately there) so the strip does not grow for the whole
  /// session.
  #[cfg_attr(feature = "tauri", allow(dead_code))]
  Remove(String),
}

/// Applies `action` to `items`, returning the new queue.
#[must_use]
pub fn reduce(items: Vec<Download>, action: DownloadAction) -> Vec<Download> {
  match action {
    DownloadAction::Push(item) => {
      let mut items = items;
      items.push(item);
      items
    }
    DownloadAction::UnShift(item) => {
      let mut next = Vec::with_capacity(items.len() + 1);
      next.push(item);
      next.extend_from_slice(&items);
      next
    }
    DownloadAction::Update(progress) => {
      let mut items = items;
      if let Some(item) = items.iter_mut().find(|item| item.url == progress.url) {
        item.percent = progress.percent * 100.0;
        item.status = progress.status;
      }
      items
    }
    DownloadAction::Remove(url) => {
      let mut items = items;
      items.retain(|item| item.url != url);
      items
    }
  }
}

/// Status of the download with the given `url`, defaulting to
/// [`ImageState::Pending`] when it is not queued.
#[must_use]
pub fn status_of(items: &[Download], url: &str) -> ImageState {
  items
    .iter()
    .find(|item| item.url == url)
    .map_or(ImageState::Pending, |item| item.status.clone())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn download(url: &str) -> Download {
    Download {
      preview: String::new(),
      url: url.to_string(),
      name: format!("{url}.jpg"),
      percent: 0.0,
      status: ImageState::Pending,
    }
  }

  #[test]
  fn queue_starts_empty() {
    assert!(Vec::<Download>::new().is_empty());
  }

  #[test]
  fn push_appends_in_order() {
    let items = reduce(Vec::new(), DownloadAction::Push(download("a")));
    let items = reduce(items, DownloadAction::Push(download("b")));
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].url, "a");
    assert_eq!(items[1].url, "b");
  }

  #[test]
  fn unshift_prepends() {
    let items = reduce(Vec::new(), DownloadAction::Push(download("a")));
    let items = reduce(items, DownloadAction::UnShift(download("b")));
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].url, "b", "unshift should prepend");
    assert_eq!(items[1].url, "a");
  }

  #[test]
  fn update_scales_percent_and_sets_status() {
    let items = reduce(Vec::new(), DownloadAction::Push(download("url1")));
    let items = reduce(
      items,
      DownloadAction::Update(DownloadProgress {
        url: "url1".to_string(),
        percent: 0.5,
        status: ImageState::Loaded,
      }),
    );
    assert_eq!(items[0].percent, 50.0, "percent should be scaled by 100");
    assert_eq!(items[0].status, ImageState::Loaded);
  }

  #[test]
  fn update_of_unknown_url_is_a_noop() {
    let items = reduce(Vec::new(), DownloadAction::Push(download("url1")));
    let items = reduce(
      items,
      DownloadAction::Update(DownloadProgress {
        url: "url_missing".to_string(),
        percent: 1.0,
        status: ImageState::Error,
      }),
    );
    assert_eq!(items[0].percent, 0.0, "should not change");
    assert_eq!(items[0].status, ImageState::Pending);
  }

  #[test]
  fn status_of_falls_back_to_pending() {
    let items = vec![download("a")];
    assert_eq!(status_of(&items, "a"), ImageState::Pending);
    assert_eq!(status_of(&items, "missing"), ImageState::Pending);
  }

  #[test]
  fn remove_drops_only_the_matching_entry() {
    let items = reduce(Vec::new(), DownloadAction::Push(download("a")));
    let items = reduce(items, DownloadAction::Push(download("b")));
    let items = reduce(items, DownloadAction::Remove("a".to_string()));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].url, "b");
  }

  #[test]
  fn remove_of_unknown_url_is_a_noop() {
    let items = reduce(Vec::new(), DownloadAction::Push(download("a")));
    let items = reduce(items, DownloadAction::Remove("missing".to_string()));
    assert_eq!(items.len(), 1, "unknown urls must not remove anything");
  }
}
