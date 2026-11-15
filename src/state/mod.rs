//! Application state: reactive store plus the pure reducers behind it.

pub mod app_state;
pub mod downloads;
pub mod i18n;
pub mod notices;
pub mod page;
pub mod theme;

pub use app_state::{AppState, use_app_state};
pub use downloads::DownloadAction;
pub use i18n::{I18n, Language, Msg};
pub use notices::{Notice, NoticeAction, NoticeState, NoticeTag};
pub use page::{Page, PageAction};
pub use theme::ThemeColor;
