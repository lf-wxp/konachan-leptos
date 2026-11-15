//! Pagination state and its pure reducer.

/// Actions that can be applied to [`Page`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageAction {
  /// Go to the previous page (no-op on the first page).
  Prev,
  /// Go to the next page (no-op on the last page).
  Next,
  /// Replace the total page count, keeping the current page.
  Total(usize),
  /// Jump to the given page, if it is inside `1..=total`.
  Invoke(usize),
  /// Return to the first page and forget the known total.
  ///
  /// Used when the query itself changes (the tag filter): the old pagination
  /// belongs to the previous result set, and `Invoke(1)` cannot be used for it
  /// because `total` may still be zero before the new response lands.
  Reset,
}

/// Current page and total page count.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Page {
  /// Currently displayed page, 1-based.
  pub current: usize,
  /// Total number of pages available.
  pub total: usize,
}

impl Default for Page {
  fn default() -> Self {
    Self {
      current: 1,
      total: 0,
    }
  }
}

impl Page {
  /// Applies `action`, returning the new state.
  ///
  /// The reducer is total: out-of-range jumps and moves past the boundaries
  /// leave the state untouched.
  #[must_use]
  pub fn reduce(self, action: PageAction) -> Self {
    match action {
      PageAction::Next => {
        if self.current < self.total {
          Self {
            current: self.current + 1,
            ..self
          }
        } else {
          self
        }
      }
      PageAction::Prev => {
        if self.current > 1 {
          Self {
            current: self.current - 1,
            ..self
          }
        } else {
          self
        }
      }
      // A shrinking result set must not leave the viewer on a page that no
      // longer exists: clamp instead of keeping the stale `current`.
      PageAction::Total(total) => Self {
        total,
        current: self.current.clamp(1, total.max(1)),
      },
      PageAction::Invoke(current) => {
        if (1..=self.total).contains(&current) {
          Self { current, ..self }
        } else {
          self
        }
      }
      PageAction::Reset => Self::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn page(current: usize, total: usize) -> Page {
    Page { current, total }
  }

  #[test]
  fn default_page_starts_at_one_without_total() {
    let p = Page::default();
    assert_eq!(p.current, 1);
    assert_eq!(p.total, 0);
  }

  #[test]
  fn next_increments_current_page() {
    let p = page(1, 10).reduce(PageAction::Next);
    assert_eq!(p.current, 2);
  }

  #[test]
  fn next_at_last_page_stays() {
    let p = page(10, 10).reduce(PageAction::Next);
    assert_eq!(p.current, 10, "should not exceed total");
  }

  #[test]
  fn prev_decrements_current_page() {
    let p = page(5, 10).reduce(PageAction::Prev);
    assert_eq!(p.current, 4);
  }

  #[test]
  fn prev_at_first_page_stays() {
    let p = page(1, 10).reduce(PageAction::Prev);
    assert_eq!(p.current, 1, "should not go below 1");
  }

  #[test]
  fn total_updates_without_touching_current() {
    let p = page(1, 0).reduce(PageAction::Total(100));
    assert_eq!(p.total, 100);
    assert_eq!(p.current, 1, "current should not change");
  }

  #[test]
  fn invoke_accepts_valid_page() {
    let p = page(1, 10).reduce(PageAction::Invoke(5));
    assert_eq!(p.current, 5);
  }

  #[test]
  fn invoke_rejects_out_of_range_page() {
    let p = page(1, 10).reduce(PageAction::Invoke(0));
    assert_eq!(p.current, 1, "page 0 is invalid");

    let p = page(1, 10).reduce(PageAction::Invoke(11));
    assert_eq!(p.current, 1, "page 11 exceeds total");
  }

  #[test]
  fn invoke_accepts_boundary_pages() {
    assert_eq!(page(5, 10).reduce(PageAction::Invoke(1)).current, 1);
    assert_eq!(page(5, 10).reduce(PageAction::Invoke(10)).current, 10);
  }

  #[test]
  fn invoke_is_rejected_before_total_is_known() {
    let p = page(1, 0).reduce(PageAction::Invoke(3));
    assert_eq!(p.current, 1, "no page is reachable while total is 0");
  }

  #[test]
  fn reset_returns_to_the_first_page_and_forgets_the_total() {
    let p = page(7, 20).reduce(PageAction::Reset);
    assert_eq!(p.current, 1);
    assert_eq!(p.total, 0);
  }

  #[test]
  fn total_clamps_a_current_page_that_no_longer_exists() {
    let p = page(9, 20).reduce(PageAction::Total(3));
    assert_eq!(p.current, 3, "must not stay past the last page");
    assert_eq!(p.total, 3);
  }

  #[test]
  fn total_of_zero_keeps_a_reachable_current_page() {
    let p = page(9, 20).reduce(PageAction::Total(0));
    assert_eq!(p.current, 1, "page 1 is the only reachable page");
    assert_eq!(p.total, 0);
  }

  #[test]
  fn total_keeps_a_valid_current_page() {
    let p = page(4, 20).reduce(PageAction::Total(10));
    assert_eq!(p.current, 4);
    assert_eq!(p.total, 10);
  }
}
