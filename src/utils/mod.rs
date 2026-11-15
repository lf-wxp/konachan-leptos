//! Browser helpers, layout maths and canvas animations.

pub mod config;
pub mod dom;
pub mod particle_progress;
pub mod pointline;
pub mod resize;
pub mod timer;
pub mod waterfall;

pub use config::is_safe;
pub use dom::{bare_rgb, get_html_image_to_vec, get_window, query_selector, random, target_as};
#[cfg(not(feature = "tauri"))]
pub use dom::{download_file, register_service_worker};
pub use particle_progress::ParticleProgress;
pub use pointline::PointLine;
pub use resize::WidthObserver;
pub use waterfall::{LayoutTile, WaterfallParams, calc_waterfall};
