//! Application shell: providers plus the lightbox stage.

use leptos::prelude::*;

use crate::components::{Background, DynamicWallpaper, Loader, Notify, Service, Stage};
use crate::state::AppState;
#[cfg(not(feature = "tauri"))]
use crate::utils::register_service_worker;

/// Root component.
#[component]
pub fn App() -> impl IntoView {
  provide_context(AppState::new());

  #[cfg(not(feature = "tauri"))]
  register_service_worker();

  view! {
    <Service/>
    <section class="lx-app">
      <Background/>
      <div class="lx-scrim" aria-hidden="true"></div>
      <DynamicWallpaper/>
      <Stage/>
      <Loader/>
    </section>
    <Notify/>
  }
}
