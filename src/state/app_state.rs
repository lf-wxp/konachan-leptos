//! Global application state, shared through the Leptos context API.

use leptos::prelude::*;
use rand::{rng, seq::SliceRandom};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::model::{Download, Image, ImageState, Mode};
use crate::utils::{LayoutTile, WaterfallParams, calc_waterfall};

use super::downloads::DownloadAction;
use super::{
  I18n, Notice, NoticeAction, NoticeTag, Page, PageAction, ThemeColor, downloads, notices,
};

/// Widest column the waterfall layout will produce, in CSS pixels.
const MAX_COLUMN_WIDTH: f64 = 300.0;
/// Narrowest column the waterfall layout will produce, in CSS pixels.
const MIN_COLUMN_WIDTH: f64 = 200.0;

/// Reactive application state.
///
/// The struct is `Copy`: every field is either a signal handle or a small
/// plain value, so it can be captured by event handlers and effects without
/// wrapping it in `Rc`.
#[derive(Clone, Copy)]
pub struct AppState {
  /// Posts of the current page, already shuffled.
  ///
  /// Posts are shared behind an [`Arc`]: the layout, the tiles and the
  /// download handler all reference the same allocation, so re-reading the
  /// list costs one pointer bump instead of cloning every string.
  pub images: RwSignal<Vec<Arc<Image>>>,
  /// Posts positioned by the waterfall layout.
  ///
  /// Cheap to read: the memo returns a shared slice.
  pub layout: Memo<Arc<[LayoutTile]>>,
  /// Current page, 1-based.
  ///
  /// Kept separate from [`AppState::page_total`] on purpose: grid effects track
  /// individual signals, so storing both fields in a single signal would
  /// re-trigger the post request every time the total page count changes.
  pub page_current: RwSignal<usize>,
  /// Total number of pages reported by the backend.
  pub page_total: RwSignal<usize>,
  /// Active tag filter.
  pub tags: RwSignal<String>,
  /// Post source format requested from the backend.
  pub mode: RwSignal<Mode>,
  /// Toggled to re-run the current query without changing page or tags.
  pub refresh: RwSignal<bool>,
  /// Whether a post list request is in flight.
  pub loading: RwSignal<bool>,
  /// Whether posts must pass the content filter (`true` = filter enabled).
  pub security: RwSignal<bool>,
  /// Width of the list container, in CSS pixels.
  pub size: RwSignal<f64>,
  /// Theme colours extracted from the background image.
  pub theme: RwSignal<ThemeColor>,
  /// Post chosen as the full-bleed wallpaper: its id and image URL.
  ///
  /// `None` keeps the random built-in background.
  pub wallpaper: RwSignal<Option<(u32, String)>>,
  /// Download queue shown in the side panel.
  pub downloads: RwSignal<Vec<Download>>,
  /// Position of every download in [`AppState::downloads`], keyed by URL.
  ///
  /// Only the URL and the index are copied, so a progress tick rebuilds a map
  /// of small values instead of cloning the whole queue (two `String`s per
  /// entry) into it.
  pub download_index: Memo<HashMap<String, usize>>,
  /// Notification list rendered in the top layer.
  pub notices: RwSignal<Vec<Notice>>,
  /// Translation table. Updated when the viewer switches language.
  pub i18n: RwSignal<I18n>,
}

impl AppState {
  /// Creates the state together with its derived waterfall layout.
  ///
  /// Must be called inside a reactive owner (e.g. a component body), because
  /// it creates a [`Memo`].
  #[must_use]
  pub fn new() -> Self {
    let images = RwSignal::new(Vec::new());
    let security = RwSignal::new(true);
    let size = RwSignal::new(0.0_f64);
    let layout = Memo::new(move |_| {
      // `with` borrows the list instead of cloning it into the layout.
      Arc::from(images.with(|items| {
        calc_waterfall(WaterfallParams {
          security: security.get(),
          width: size.get(),
          max_width: MAX_COLUMN_WIDTH,
          min_width: MIN_COLUMN_WIDTH,
          images: items,
        })
      }))
    });

    let downloads: RwSignal<Vec<Download>> = RwSignal::new(Vec::new());
    let download_index = Memo::new(move |_| {
      downloads.with(|items| {
        items
          .iter()
          .enumerate()
          .map(|(index, item)| (item.url.clone(), index))
          .collect()
      })
    });

    Self {
      images,
      layout,
      downloads,
      download_index,
      page_current: RwSignal::new(Page::default().current),
      page_total: RwSignal::new(Page::default().total),
      tags: RwSignal::new(String::new()),
      mode: RwSignal::new(Mode::default()),
      refresh: RwSignal::new(true),
      loading: RwSignal::new(false),
      security,
      size,
      theme: RwSignal::new(ThemeColor::default()),
      wallpaper: RwSignal::new(None),
      notices: RwSignal::new(Vec::new()),
      i18n: RwSignal::new(I18n::detect()),
    }
  }

  /// Replaces the post list, shuffling it before it is laid out.
  pub fn set_images(&self, mut images: Vec<Image>) {
    images.shuffle(&mut rng());
    self
      .images
      .set(images.into_iter().map(Arc::new).collect::<Vec<_>>());
  }

  /// Current page and total page count, as a single value.
  #[must_use]
  pub fn page(&self) -> Page {
    Page {
      current: self.page_current.get_untracked(),
      total: self.page_total.get_untracked(),
    }
  }

  /// Replaces the tag filter, ignoring writes that keep the current value.
  ///
  /// The query effect tracks the tag signal directly, so an identical write
  /// would trigger a redundant request (the previous dependency tuple was
  /// compared by value).
  pub fn set_tags(&self, tags: String) {
    if self.tags.get_untracked() != tags {
      self.tags.set(tags);
    }
  }

  /// Applies a new tag filter and resets the pagination in one batch.
  ///
  /// Both writes belong to the same user action; without the batch the query
  /// effect would fire once for the page reset and once for the tags, i.e. two
  /// requests per search (one of which is immediately superseded).
  pub fn set_query(&self, tags: String) {
    batch(move || {
      self.dispatch_page(PageAction::Reset);
      self.tags.set(tags);
    });
  }

  /// Applies a pagination action.
  ///
  /// Only the signals that actually change are written, so consumers of one
  /// field are not notified when the other one changes.
  pub fn dispatch_page(&self, action: PageAction) {
    let current = self.page();
    let next = current.reduce(action);
    if next.current != current.current {
      self.page_current.set(next.current);
    }
    if next.total != current.total {
      self.page_total.set(next.total);
    }
  }

  /// Applies a download queue action.
  pub fn dispatch_download(&self, action: DownloadAction) {
    self
      .downloads
      .update(|items| *items = downloads::reduce(std::mem::take(items), action));
  }

  /// Queues `image` for download, returning `false` when it is already queued.
  ///
  /// Shared by every transport. Browser builds receive no backend progress
  /// events, but the queue entry is what disables the download button and
  /// keeps a failed transfer retryable from the strip.
  pub fn queue_download(&self, image: &Image) -> bool {
    if self.is_queued(&image.url) {
      return false;
    }
    self.dispatch_download(DownloadAction::UnShift(Download {
      preview: image.preview.clone(),
      url: image.url.clone(),
      name: image.name.clone(),
      percent: 0.0,
      status: ImageState::Pending,
    }));
    true
  }

  /// Whether `url` is already queued for download.
  #[must_use]
  pub fn is_queued(&self, url: &str) -> bool {
    self.download_index.with(|index| index.contains_key(url))
  }

  /// Queue entry of `url`, if it is being downloaded.
  #[must_use]
  pub fn download_of(&self, url: &str) -> Option<Download> {
    let position = self.download_index.with(|index| index.get(url).copied())?;
    self.downloads.with(|items| items.get(position).cloned())
  }

  /// Shows a notification, auto-dismissing it after `duration` seconds.
  pub fn notify(&self, content: impl Into<String>, tag: NoticeTag, duration: Option<u32>) {
    let notice = Notice::new(content.into(), tag, duration);
    if let Some(seconds) = duration {
      let id = notice.id.clone();
      let state = *self;
      set_timeout(
        move || state.pre_remove_notice(&id),
        Duration::from_secs(u64::from(seconds)),
      );
    }
    self.notices.update(|list| {
      *list = notices::reduce(std::mem::take(list), NoticeAction::Append(notice));
    });
  }

  /// Marks a notification as fading out.
  pub fn pre_remove_notice(&self, id: &str) {
    self.notices.update(|list| {
      *list = notices::reduce(
        std::mem::take(list),
        NoticeAction::PreRemove(id.to_string()),
      );
    });
  }

  /// Drops a notification immediately.
  pub fn remove_notice(&self, id: &str) {
    self.notices.update(|list| {
      *list = notices::reduce(std::mem::take(list), NoticeAction::Remove(id.to_string()));
    });
  }
}

impl Default for AppState {
  fn default() -> Self {
    Self::new()
  }
}

/// Reads the shared [`AppState`] from the component context.
///
/// # Panics
///
/// Panics when no [`AppState`] was provided, which can only happen if a
/// component is used outside of the application root.
#[must_use]
pub fn use_app_state() -> AppState {
  expect_context::<AppState>()
}
