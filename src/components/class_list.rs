//! Helper for building class attributes.

/// Joins `base` with the enabled modifiers, without duplicated separators.
///
/// String concatenation of conditional classes would leave trailing spaces
/// whenever a modifier is disabled; the previous framework normalised the
/// attribute, so the same normalisation is applied here.
///
/// # Examples
///
/// ```
/// assert_eq!(class_list("item", [Some("current"), None]), "item current");
/// assert_eq!(class_list("item", [None, None]), "item");
/// ```
#[must_use]
pub fn class_list<'a>(
  base: &'a str,
  modifiers: impl IntoIterator<Item = Option<&'a str>>,
) -> String {
  let mut class = String::from(base);
  for modifier in modifiers.into_iter().flatten() {
    class.push(' ');
    class.push_str(modifier);
  }
  class
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn keeps_only_enabled_modifiers() {
    assert_eq!(class_list("item", [None, Some("active")]), "item active");
  }

  #[test]
  fn returns_the_base_when_nothing_is_enabled() {
    assert_eq!(class_list("item", [None, None]), "item");
  }

  #[test]
  fn accepts_an_empty_modifier_list() {
    let modifiers: [Option<&str>; 0] = [];
    assert_eq!(class_list("item", modifiers), "item");
  }
}
