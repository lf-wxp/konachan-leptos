//! Waterfall (masonry) layout computation.
//!
//! The layout is a pure function of the container width, the number of
//! columns and the image list, which keeps it trivially testable and lets the
//! UI memoise the result.

use std::sync::Arc;

use crate::model::Image;

/// A post placed by the layout.
///
/// The post is shared through an [`Arc`] and the presentation is kept as plain
/// numbers, so a memoised layout can be cloned and read without copying any of
/// the (string heavy) post data or formatting inline CSS.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutTile {
  /// Post data, shared with the image list.
  pub image: Arc<Image>,
  /// Display index, which drives the staggered animation delays.
  pub index: usize,
  /// Left edge, in CSS pixels.
  pub x: f64,
  /// Top edge, in CSS pixels.
  pub y: f64,
  /// Rendered width, in CSS pixels.
  pub width: f64,
  /// Rendered height, in CSS pixels.
  pub height: f64,
}

/// Index of the shortest column and its height.
fn shortest_column(columns: &[f64]) -> (f64, usize) {
  columns
    .iter()
    .enumerate()
    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    .map(|(index, &height)| (height, index))
    .unwrap_or((0.0, 0))
}

/// Column count and column width for a container of `width` pixels.
fn calc_column_width(max_width: f64, min_width: f64, width: f64) -> (usize, f64) {
  if width <= 0.0 {
    return (0, 0.0);
  }

  let col_width = if width % max_width > 0.0 {
    width / (width / max_width).ceil()
  } else {
    max_width
  };

  if col_width < min_width {
    (1, width)
  } else {
    ((width / max_width).ceil() as usize, col_width)
  }
}

/// Adds `height` to the shortest column.
fn add_to_shortest_column(height: f64, columns: &mut [f64]) {
  if !columns.is_empty() {
    let (_, index) = shortest_column(columns);
    columns[index] += height;
  }
}

/// Top-left corner of the next item in the shortest column.
fn calc_position(col_width: f64, columns: &[f64]) -> (f64, f64) {
  let (min, index) = shortest_column(columns);
  (index as f64 * col_width, min)
}

/// Height of `image` once scaled to `col_width`, preserving its aspect ratio.
///
/// Falls back to a square cell when the backend reports a non-positive width,
/// which would otherwise produce `NaN` and drop the tile out of the layout.
fn scaled_height(image: &Image, col_width: f64) -> f64 {
  if image.width > 0.0 {
    col_width * (image.height / image.width)
  } else {
    col_width
  }
}

/// Positions a single item, preserving its aspect ratio.
fn lay_out_item(image: &Arc<Image>, index: usize, col_width: f64, columns: &[f64]) -> LayoutTile {
  let height = scaled_height(image, col_width);
  let (x, y) = calc_position(col_width, columns);
  LayoutTile {
    image: Arc::clone(image),
    index,
    x,
    y,
    width: col_width,
    height,
  }
}

/// Lays out `images`, optionally dropping posts that failed the content filter.
fn update_layout(
  security: bool,
  col_width: f64,
  columns: &mut [f64],
  images: &[Arc<Image>],
) -> Vec<LayoutTile> {
  images
    .iter()
    .filter(|image| !security || image.security)
    .enumerate()
    .map(|(index, image)| {
      let tile = lay_out_item(image, index, col_width, columns);
      add_to_shortest_column(tile.height, columns);
      tile
    })
    .collect()
}

/// Inputs of [`calc_waterfall`].
pub struct WaterfallParams<'a> {
  /// When `true`, only posts that passed the content filter are kept.
  pub security: bool,
  /// Container width, in CSS pixels.
  pub width: f64,
  /// Preferred maximum column width.
  pub max_width: f64,
  /// Minimum column width before falling back to a single column.
  pub min_width: f64,
  /// Images to lay out, in display order.
  pub images: &'a [Arc<Image>],
}

/// Computes absolute positions for every image.
///
/// Returns an empty list when the container has no width yet, which is the
/// case before the first resize observation.
#[must_use]
pub fn calc_waterfall(params: WaterfallParams<'_>) -> Vec<LayoutTile> {
  let WaterfallParams {
    security,
    width,
    max_width,
    min_width,
    images,
  } = params;
  let (column, col_width) = calc_column_width(max_width, min_width, width);
  if column == 0 {
    return Vec::new();
  }
  let mut columns = vec![0.0_f64; column];
  update_layout(security, col_width, &mut columns, images)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn image(id: u32, width: f64, height: f64, security: bool) -> Arc<Image> {
    Arc::new(Image {
      id,
      width,
      height,
      security,
      sample_width: width,
      sample_height: height,
      preview_width: width,
      preview_height: height,
      name: format!("img_{id}"),
      ..Image::default()
    })
  }

  fn layout(security: bool, width: f64, images: &[Arc<Image>]) -> Vec<LayoutTile> {
    calc_waterfall(WaterfallParams {
      security,
      width,
      max_width: 300.0,
      min_width: 200.0,
      images,
    })
  }

  #[test]
  fn shortest_column_of_empty_layout_is_first() {
    assert_eq!(shortest_column(&[]), (0.0, 0));
  }

  #[test]
  fn shortest_column_of_single_column_is_that_column() {
    assert_eq!(shortest_column(&[100.0]), (100.0, 0));
  }

  #[test]
  fn shortest_column_picks_the_smallest_height() {
    assert_eq!(shortest_column(&[300.0, 100.0, 200.0]), (100.0, 1));
  }

  #[test]
  fn column_width_of_zero_container_is_empty() {
    assert_eq!(calc_column_width(300.0, 200.0, 0.0), (0, 0.0));
  }

  #[test]
  fn column_width_of_negative_container_is_empty() {
    assert_eq!(calc_column_width(300.0, 200.0, -100.0), (0, 0.0));
  }

  #[test]
  fn column_width_splits_exact_multiples() {
    assert_eq!(calc_column_width(300.0, 200.0, 600.0), (2, 300.0));
  }

  #[test]
  fn column_width_falls_back_to_single_column_when_narrow() {
    assert_eq!(calc_column_width(300.0, 200.0, 150.0), (1, 150.0));
  }

  #[test]
  fn waterfall_without_images_is_empty() {
    assert!(layout(false, 600.0, &[]).is_empty());
  }

  #[test]
  fn waterfall_without_width_is_empty() {
    let images = [image(1, 100.0, 200.0, true)];
    assert!(layout(false, 0.0, &images).is_empty());
  }

  #[test]
  fn waterfall_positions_a_single_image() {
    let images = [image(1, 100.0, 200.0, true)];
    let result = layout(false, 300.0, &images);

    assert_eq!(result.len(), 1);
    let tile = &result[0];
    assert_eq!((tile.x, tile.y), (0.0, 0.0));
    assert_eq!((tile.width, tile.height), (300.0, 600.0));
    assert_eq!(tile.index, 0);
    assert_eq!(tile.image.id, 1);
  }

  #[test]
  fn waterfall_preserves_aspect_ratio() {
    let images = [image(1, 100.0, 200.0, true)];
    let tile = &layout(false, 300.0, &images)[0];
    let ratio = tile.height / tile.width;
    assert!((ratio - 2.0).abs() < 0.001, "original ratio is 200/100");
  }

  #[test]
  fn waterfall_treats_a_zero_width_post_as_square() {
    let images = [image(1, 0.0, 200.0, true)];
    let tile = &layout(false, 300.0, &images)[0];
    assert_eq!(tile.height, 300.0);
    assert!(tile.height.is_finite());
  }

  #[test]
  fn waterfall_applies_the_content_filter() {
    let images = [
      image(1, 100.0, 100.0, true),
      image(2, 100.0, 100.0, false),
      image(3, 100.0, 100.0, true),
    ];

    let filtered = layout(true, 600.0, &images);
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|tile| tile.image.security));
    // Display indices are renumbered after filtering.
    assert_eq!(
      filtered.iter().map(|tile| tile.index).collect::<Vec<_>>(),
      vec![0, 1]
    );

    let unfiltered = layout(false, 600.0, &images);
    assert_eq!(unfiltered.len(), 3);
  }

  #[test]
  fn waterfall_distributes_items_across_columns() {
    let images: Vec<Arc<Image>> = (0..4).map(|id| image(id, 100.0, 100.0, true)).collect();
    let result = layout(false, 600.0, &images);

    assert_eq!(result.len(), 4);
    let positions: Vec<(f64, f64)> = result.iter().map(|tile| (tile.x, tile.y)).collect();
    assert_eq!(
      positions,
      vec![(0.0, 0.0), (300.0, 0.0), (0.0, 300.0), (300.0, 300.0),]
    );
  }
}
