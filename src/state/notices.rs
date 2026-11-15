//! Toast notifications: model plus pure reducer.

use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic id source, unique for the lifetime of the page.
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Severity of a notification, which selects its icon and colour.
///
/// Only [`NoticeTag::Danger`] is emitted by the current UI; the other variants
/// are kept so new call sites do not need to change the model.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NoticeTag {
  /// Neutral information.
  Info,
  /// Operation completed.
  Success,
  /// Something looks suspicious.
  Warning,
  /// Operation failed.
  Danger,
}

/// Visual state of a notification.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NoticeState {
  /// Visible.
  Exist,
  /// Fading out; removed once the exit animation finishes.
  Perish,
}

/// A single notification.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Notice {
  /// Unique identifier.
  pub id: String,
  /// Message shown to the user.
  pub content: String,
  /// Severity driving the icon and colour.
  pub tag: NoticeTag,
  /// Auto-dismiss delay, in seconds.
  pub duration: Option<u32>,
  /// Current visual state.
  pub state: NoticeState,
}

impl Notice {
  /// Creates a notification in the [`NoticeState::Exist`] state.
  #[must_use]
  pub fn new(content: String, tag: NoticeTag, duration: Option<u32>) -> Self {
    Self {
      id: NEXT_ID.fetch_add(1, Ordering::Relaxed).to_string(),
      content,
      tag,
      duration,
      state: NoticeState::Exist,
    }
  }
}

/// Actions that can be applied to the notification list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoticeAction {
  /// Append a notification.
  Append(Notice),
  /// Drop a notification immediately.
  Remove(String),
  /// Mark a notification as fading out.
  PreRemove(String),
}

/// Applies `action` to `list`, returning the new list.
#[must_use]
pub fn reduce(mut list: Vec<Notice>, action: NoticeAction) -> Vec<Notice> {
  match action {
    NoticeAction::Append(notice) => list.push(notice),
    NoticeAction::Remove(id) => list.retain(|notice| notice.id != id),
    NoticeAction::PreRemove(id) => {
      if let Some(notice) = list.iter_mut().find(|notice| notice.id == id) {
        notice.state = NoticeState::Perish;
      }
    }
  }
  list
}

#[cfg(test)]
mod tests {
  use super::*;

  fn notice(content: &str) -> Notice {
    Notice::new(content.to_string(), NoticeTag::Info, None)
  }

  #[test]
  fn ids_are_unique() {
    let first = notice("a");
    let second = notice("b");
    assert_ne!(first.id, second.id);
  }

  #[test]
  fn new_notice_is_visible() {
    assert_eq!(notice("a").state, NoticeState::Exist);
  }

  #[test]
  fn append_adds_to_the_end() {
    let list = reduce(Vec::new(), NoticeAction::Append(notice("a")));
    let list = reduce(list, NoticeAction::Append(notice("b")));
    assert_eq!(list.len(), 2);
    assert_eq!(list[1].content, "b");
  }

  #[test]
  fn pre_remove_marks_notice_as_perish() {
    let item = notice("a");
    let id = item.id.clone();
    let list = reduce(Vec::new(), NoticeAction::Append(item));
    let list = reduce(list, NoticeAction::PreRemove(id));
    assert_eq!(list.len(), 1, "the notice stays until the animation ends");
    assert_eq!(list[0].state, NoticeState::Perish);
  }

  #[test]
  fn remove_drops_the_notice() {
    let item = notice("a");
    let id = item.id.clone();
    let list = reduce(Vec::new(), NoticeAction::Append(item));
    let list = reduce(list, NoticeAction::Remove(id));
    assert!(list.is_empty());
  }

  #[test]
  fn actions_on_unknown_ids_are_noops() {
    let list = reduce(Vec::new(), NoticeAction::Append(notice("a")));
    let list = reduce(list, NoticeAction::PreRemove("missing".to_string()));
    assert_eq!(list[0].state, NoticeState::Exist);

    let list = reduce(list, NoticeAction::Remove("missing".to_string()));
    assert_eq!(list.len(), 1, "unknown ids must not remove anything");
  }
}
