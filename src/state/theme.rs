//! Theme colours extracted from the current background image.

/// Colours driving the CSS custom properties of the application.
#[derive(PartialEq, Clone, Debug)]
pub struct ThemeColor {
  /// Base colour, as a bare `r, g, b` triplet.
  pub theme_color: String,
  /// Primary accent, as a bare `r, g, b` triplet.
  pub primary_color: String,
  /// Ancillary accent, as a bare `r, g, b` triplet.
  pub ancillary_color: String,
}

impl ThemeColor {
  /// Builds a theme from three bare `r, g, b` triplets.
  #[must_use]
  pub fn new(theme: String, primary: String, ancillary: String) -> Self {
    Self {
      theme_color: theme,
      primary_color: primary,
      ancillary_color: ancillary,
    }
  }

  /// The CSS custom properties, as `(name, value)` pairs.
  ///
  /// Applied one property at a time so that writing the theme does not wipe
  /// unrelated inline styles of the `<html>` element, which `set_css_text`
  /// would do.
  #[must_use]
  pub fn declarations(&self) -> [(&'static str, String); 4] {
    let Self {
      theme_color,
      primary_color,
      ancillary_color,
    } = self;
    [
      ("--theme-base-color", format!("rgb({theme_color})")),
      ("--theme-base-color-rgb", theme_color.clone()),
      ("--theme-primary-color", format!("rgb({primary_color})")),
      ("--theme-ancillary-color", format!("rgb({ancillary_color})")),
    ]
  }

  /// The three colours as `rgb()` strings.
  #[must_use]
  pub fn colors(&self) -> (String, String, String) {
    let Self {
      theme_color,
      primary_color,
      ancillary_color,
    } = self;
    (
      format!("rgb({theme_color})"),
      format!("rgb({primary_color})"),
      format!("rgb({ancillary_color})"),
    )
  }
}

impl Default for ThemeColor {
  fn default() -> Self {
    // Every triplet must be a valid colour: these values are written to the
    // `<html>` element before the background image has been sampled, and the
    // empty string used to produce `rgb()`, which is invalid at computed-value
    // time and silently dropped the whole declaration (invisible loaders).
    Self {
      theme_color: "57, 204, 204".to_string(),
      primary_color: "57, 204, 204".to_string(),
      ancillary_color: "57, 204, 204".to_string(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_theme_is_teal() {
    assert_eq!(ThemeColor::default().theme_color, "57, 204, 204");
  }

  #[test]
  fn default_theme_emits_only_valid_colours() {
    for (name, value) in ThemeColor::default().declarations() {
      assert!(!value.is_empty(), "{name} must not be empty");
      assert_ne!(value, "rgb()", "{name} must not be an empty rgb()");
    }
  }

  #[test]
  fn declarations_cover_all_four_variables() {
    let declarations =
      ThemeColor::new("1, 2, 3".into(), "4, 5, 6".into(), "7, 8, 9".into()).declarations();
    assert_eq!(
      declarations,
      [
        ("--theme-base-color", "rgb(1, 2, 3)".to_string()),
        ("--theme-base-color-rgb", "1, 2, 3".to_string()),
        ("--theme-primary-color", "rgb(4, 5, 6)".to_string()),
        ("--theme-ancillary-color", "rgb(7, 8, 9)".to_string()),
      ]
    );
  }

  #[test]
  fn colors_are_rgb_strings() {
    let (base, primary, ancillary) =
      ThemeColor::new("1, 2, 3".into(), "4, 5, 6".into(), "7, 8, 9".into()).colors();
    assert_eq!(base, "rgb(1, 2, 3)");
    assert_eq!(primary, "rgb(4, 5, 6)");
    assert_eq!(ancillary, "rgb(7, 8, 9)");
  }
}
