//! Minimal internationalisation layer (English and Chinese).

/// Language used by the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
  /// English.
  En,
  /// Simplified Chinese.
  Zh,
}

impl Language {
  /// Maps a browser language tag (`en-US`, `zh-CN`, …) to a supported
  /// language, falling back to English.
  #[must_use]
  pub fn from_tag(tag: &str) -> Self {
    match tag.get(..2) {
      Some("zh") => Language::Zh,
      _ => Language::En,
    }
  }
}

/// Translated message keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg {
  /// Shown when the post list request fails.
  GetListError,
  /// Shown when an image download fails.
  DownloadError,
  /// Accessible name of the "previous page" control.
  PreviousPage,
  /// Accessible name of the "next page" control.
  NextPage,
  /// Accessible name prefix of a page shortcut.
  Page,
}

/// Translation table for the current language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct I18n {
  language: Language,
}

impl I18n {
  /// Creates a table for `language`.
  #[must_use]
  pub const fn new(language: Language) -> Self {
    Self { language }
  }

  /// Detects the language from `navigator.language`, defaulting to English.
  #[must_use]
  pub fn detect() -> Self {
    let tag = web_sys::window()
      .and_then(|window| window.navigator().language())
      .unwrap_or_default();
    Self::new(Language::from_tag(&tag))
  }

  /// Active language.
  #[must_use]
  pub const fn language(self) -> Language {
    self.language
  }

  /// Language tag written to the `<html lang>` attribute.
  #[must_use]
  pub const fn tag(&self) -> &'static str {
    match self.language {
      Language::En => "en",
      Language::Zh => "zh-CN",
    }
  }

  /// Translates `msg` into the current language.
  #[must_use]
  pub const fn t(&self, msg: Msg) -> &'static str {
    match (self.language, msg) {
      (Language::En, Msg::GetListError) => "Get image list error, try again!",
      (Language::Zh, Msg::GetListError) => "获取图片列表失败，请重试",
      (Language::En, Msg::DownloadError) => "Download failed, try again!",
      (Language::Zh, Msg::DownloadError) => "下载失败，请重试",
      (Language::En, Msg::PreviousPage) => "Previous page",
      (Language::Zh, Msg::PreviousPage) => "上一页",
      (Language::En, Msg::NextPage) => "Next page",
      (Language::Zh, Msg::NextPage) => "下一页",
      (Language::En, Msg::Page) => "Page",
      (Language::Zh, Msg::Page) => "页码",
    }
  }
}

impl Default for I18n {
  fn default() -> Self {
    Self::new(Language::En)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_tag_detects_supported_languages() {
    assert_eq!(Language::from_tag("en-US"), Language::En);
    assert_eq!(Language::from_tag("en"), Language::En);
    assert_eq!(Language::from_tag("zh-CN"), Language::Zh);
    assert_eq!(Language::from_tag("zh"), Language::Zh);
  }

  #[test]
  fn from_tag_falls_back_to_english() {
    assert_eq!(Language::from_tag("fr-FR"), Language::En);
    assert_eq!(Language::from_tag("ja"), Language::En);
    assert_eq!(Language::from_tag(""), Language::En);
    assert_eq!(Language::from_tag("z"), Language::En);
  }

  #[test]
  fn messages_are_translated() {
    assert_eq!(
      I18n::new(Language::En).t(Msg::GetListError),
      "Get image list error, try again!"
    );
    assert_eq!(
      I18n::new(Language::Zh).t(Msg::GetListError),
      "获取图片列表失败，请重试"
    );
  }

  #[test]
  fn download_error_is_translated() {
    assert_eq!(
      I18n::new(Language::En).t(Msg::DownloadError),
      "Download failed, try again!"
    );
    assert_eq!(
      I18n::new(Language::Zh).t(Msg::DownloadError),
      "下载失败，请重试"
    );
  }

  #[test]
  fn language_tags_match_the_detected_language() {
    assert_eq!(I18n::new(Language::En).tag(), "en");
    assert_eq!(I18n::new(Language::Zh).tag(), "zh-CN");
  }

  #[test]
  fn default_is_english() {
    assert_eq!(
      I18n::default().t(Msg::GetListError),
      "Get image list error, try again!"
    );
  }
}
