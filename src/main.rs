//! Konachan - a modern web application for browsing and downloading images.
//!
//! A Tauri and Web based client for the Konachan image board, built with
//! Leptos and WebAssembly. Features:
//! - Waterfall layout display
//! - Download management with progress tracking
//! - Animated background and wallpaper
//! - Multi-language support (English and Chinese)

mod api;
mod app;
mod components;
mod model;
mod state;
mod utils;

fn main() {
  leptos::mount::mount_to_body(app::App);
}
