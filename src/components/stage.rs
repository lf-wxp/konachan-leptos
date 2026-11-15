//! Lightbox stage: one plate, a film rail, and the controls for the current page.

use leptos::prelude::*;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use web_sys::{HtmlInputElement, KeyboardEvent, PointerEvent, WheelEvent};

#[cfg(not(all(feature = "fake", not(feature = "tauri"))))]
use crate::api::download_image;
use crate::components::{
  DownloadIcon, DownloadList, Image as ImageTile, Nav, ReelIcon, Search, Setting,
};
#[cfg(not(all(feature = "fake", not(feature = "tauri"))))]
use crate::model::DownloadProgress;
use crate::model::{Image, ImageState};
use crate::state::{DownloadAction, Language, PageAction, use_app_state};
#[cfg(not(all(feature = "fake", not(feature = "tauri"))))]
use crate::state::{Msg, NoticeTag};
use crate::utils::{WidthObserver, target_as};

/// Gap between the stage edge and the blocks, and between any two blocks.
/// Keep in sync with `--lx-gap` in `static/css/_lightbox.scss`.
const GAP: f64 = 12.0;
/// Height of the strip holding the mark and the palette chip.
const TOP_BAR: f64 = 28.0;
/// Height of the dock.
const DOCK_BAR: f64 = 48.0;
/// Top edge shared by the plate and the rail.
const CONTENT_TOP: f64 = GAP + TOP_BAR + GAP;
/// Bottom edge of the dock.
const DOCK_TOP: f64 = GAP + DOCK_BAR;
/// Bottom edge shared by the plate and the rail.
const CONTENT_BOTTOM: f64 = DOCK_TOP + GAP;

/// Which end of the next page to open after a page turn.
#[derive(Clone, Copy)]
enum Edge {
  /// First visible post.
  First,
  /// Last visible post.
  Last,
}

/// How much of the page is kept beside the plate.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Density {
  /// Two columns of slits.
  Tight,
  /// One column of slits.
  Gallery,
  /// A hairline, widened on hover.
  Cinema,
}

fn is_zh(language: Language) -> bool {
  language == Language::Zh
}

fn picture_url(image: &Image) -> String {
  if !image.sample.is_empty() {
    image.sample.clone()
  } else if !image.url.is_empty() {
    image.url.clone()
  } else {
    image.preview.clone()
  }
}

fn enqueue(state: crate::state::AppState, image: Arc<Image>) {
  if !state.queue_download(&image) {
    return;
  }

  // With `--features fake` the tile is queued and a timer walks its percentage
  // up, so the strip can be looked at without Tauri; the real download is
  // skipped so nothing lands on disk.
  #[cfg(all(feature = "fake", not(feature = "tauri")))]
  fake_download(state, &image);

  #[cfg(not(all(feature = "fake", not(feature = "tauri"))))]
  start_download(state, &image);
}

/// Hands the image to the transport and settles its queue entry afterwards.
///
/// The desktop build reports progress through backend events and never
/// finishes the transfer here; a browser build has no such events, so the
/// entry is dropped as soon as the browser accepted the download, and left in
/// [`ImageState::Error`] (with its retry button) when it did not.
#[cfg(not(all(feature = "fake", not(feature = "tauri"))))]
fn start_download(state: crate::state::AppState, image: &Image) {
  let url = image.url.clone();
  let name = image.name.clone();
  leptos::task::spawn_local(async move {
    if download_image(&url, &name).await.is_err() {
      state.dispatch_download(DownloadAction::Update(DownloadProgress {
        url: url.clone(),
        percent: 0.0,
        status: ImageState::Error,
      }));
      state.notify(
        state.i18n.get().t(Msg::DownloadError),
        NoticeTag::Danger,
        Some(3),
      );
    } else {
      #[cfg(not(feature = "tauri"))]
      state.dispatch_download(DownloadAction::Remove(url));
    }
  });
}

/// Number of progress ticks a simulated download reports.
#[cfg(all(feature = "fake", not(feature = "tauri")))]
const FAKE_STEPS: u8 = 24;

/// Reports progress on the already queued `image`, standing in for the desktop
/// backend's `download_image` command and its progress events.
#[cfg(all(feature = "fake", not(feature = "tauri")))]
fn fake_download(state: crate::state::AppState, image: &Image) {
  let url = image.url.clone();
  leptos::task::spawn_local(async move {
    for step in 1..=FAKE_STEPS {
      sleep(90).await;
      if !state.is_queued(&url) {
        return;
      }
      state.dispatch_download(DownloadAction::Update(crate::model::DownloadProgress {
        url: url.clone(),
        percent: f32::from(step) / f32::from(FAKE_STEPS),
        status: if step == FAKE_STEPS {
          ImageState::Loaded
        } else {
          ImageState::Pending
        },
      }));
    }
  });
}

/// Resolves after roughly `ms` milliseconds.
#[cfg(all(feature = "fake", not(feature = "tauri")))]
async fn sleep(ms: i32) {
  let promise = js_sys::Promise::new(&mut |resolve, _| {
    let _ = crate::utils::get_window()
      .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
  });
  let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

fn project(state: crate::state::AppState, image: &Image) {
  let url = picture_url(image);
  if url.is_empty() {
    return;
  }
  state.wallpaper.set(Some((image.id, url)));
}

fn toggle_tag(state: crate::state::AppState, tag: String, exclude: bool) {
  let token = if exclude {
    format!("-{tag}")
  } else {
    tag.clone()
  };
  let negative = format!("-{tag}");
  let mut parts: Vec<String> = state
    .tags
    .get_untracked()
    .split_whitespace()
    .map(str::to_string)
    .collect();
  let exists = parts.iter().any(|part| part == &token);
  parts.retain(|part| part != &tag && part != &negative);
  if !exists {
    parts.push(token);
  }
  state.set_query(parts.join(" "));
}

/// How long a pending page-turn hint stays armed, in seconds.
const PENDING_TIMEOUT: u64 = 5;

/// Distinguishes consecutive arming calls so a stale timeout cannot clear a
/// newer hint (two page turns within [`PENDING_TIMEOUT`]).
static PENDING_GEN: AtomicU64 = AtomicU64::new(0);

/// Arms the "which end of the new page to open" hint.
///
/// The hint is consumed by the effect that runs when the image list changes.
/// A page turn whose request fails never changes it, so without the timeout
/// the stale hint would fire on the next unrelated update and jump the plate.
fn arm_pending(pending: RwSignal<Option<Edge>>, edge: Edge) {
  pending.set(Some(edge));
  // Own this arming with a fresh generation; the timeout clears the hint only
  // when no later turn superseded it within the window.
  let ticket = PENDING_GEN.fetch_add(1, Ordering::Relaxed) + 1;
  set_timeout(
    move || {
      if PENDING_GEN.load(Ordering::Relaxed) == ticket {
        pending.set(None);
      }
    },
    Duration::from_secs(PENDING_TIMEOUT),
  );
}

fn step_plate(
  state: crate::state::AppState,
  visible: Memo<Vec<Arc<Image>>>,
  locked: RwSignal<Option<u32>>,
  scrub: RwSignal<Option<u32>>,
  pending: RwSignal<Option<Edge>>,
  dir: i32,
) {
  let items = visible.get_untracked();
  if items.is_empty() {
    return;
  }
  let current = scrub.get_untracked().or(locked.get_untracked());
  let index = items
    .iter()
    .position(|image| Some(image.id) == current)
    .unwrap_or(0);
  if dir < 0 && index == 0 {
    if state.page_current.get_untracked() > 1 {
      arm_pending(pending, Edge::Last);
      scrub.set(None);
      state.dispatch_page(PageAction::Prev);
    }
    return;
  }
  let next = if dir > 0 { index + 1 } else { index - 1 };
  if let Some(image) = items.get(next) {
    locked.set(Some(image.id));
    scrub.set(None);
    return;
  }
  if dir > 0 && state.page_current.get_untracked() < state.page_total.get_untracked() {
    arm_pending(pending, Edge::First);
    scrub.set(None);
    state.dispatch_page(PageAction::Next);
  }
}

/// The viewing room: wallpaper margins, the current plate, and the page rail.
#[component]
pub fn Stage() -> impl IntoView {
  let state = use_app_state();
  let root = NodeRef::<leptos::html::Div>::new();
  let width = RwSignal::new(1100.0);
  Effect::new(move |_| WidthObserver::new(root, width));

  let narrow = Memo::new(move |_| width.get() < 760.0);
  let density = RwSignal::new(Density::Gallery);
  let rail_hot = RwSignal::new(false);
  let locked = RwSignal::new(None::<u32>);
  let scrub = RwSignal::new(None::<u32>);
  let pending = RwSignal::new(None::<Edge>);
  let wheel_at = RwSignal::new(0.0);
  let swipe = Rc::new(std::cell::Cell::new(None::<(i32, i32)>));

  let visible = Memo::new(move |_| {
    state.layout.with(|tiles| {
      tiles
        .iter()
        .map(|tile| Arc::clone(&tile.image))
        .collect::<Vec<_>>()
    })
  });

  Effect::new(move |_| {
    let next = width.get();
    if next > 0.0 {
      state.size.set(next);
    }
  });

  Effect::new(move |_| {
    let items = visible.get();
    if let Some(edge) = pending.get_untracked() {
      pending.set(None);
      let image = match edge {
        Edge::First => items.first(),
        Edge::Last => items.last(),
      };
      locked.set(image.map(|image| image.id));
      return;
    }
    let current = locked.get_untracked();
    if current.is_some_and(|id| items.iter().any(|image| image.id == id)) {
      return;
    }
    let index = items.len().saturating_sub(1).min(10);
    locked.set(items.get(index).map(|image| image.id));
  });

  let shown = Memo::new(move |_| {
    let id = scrub.get().or(locked.get());
    visible.with(|items| items.iter().find(|image| Some(image.id) == id).cloned())
  });

  // Crossfade backdrop: the frame being replaced stays painted underneath
  // until the incoming one has faded in, so a swap never flashes the plate's
  // own background.
  let under = RwSignal::new(None::<String>);
  Effect::new(move |previous: Option<String>| -> String {
    let Some(image) = shown.get() else {
      under.set(None);
      return String::new();
    };
    if let Some(last) = previous.filter(|last| !last.is_empty()) {
      under.set(Some(last));
    }
    picture_url(&image)
  });

  // The rail only overflows on the narrow layout, where it scrolls sideways.
  // Whenever the plate moves on its own (keys, swipe, click, committed hover),
  // the matching slit is brought into view so it is never lost off-screen.
  Effect::new(move |_| {
    let Some(id) = locked.get() else { return };
    leptos::task::spawn_local(async move {
      // The rail commits its rows a frame after the plate picks an image, so
      // wait for that frame before looking for the slit.
      let frame = js_sys::Promise::new(&mut |resolve, _| {
        let _ = crate::utils::get_window().request_animation_frame(&resolve);
      });
      if wasm_bindgen_futures::JsFuture::from(frame).await.is_err() {
        return;
      }
      let selector = format!(".lx-rail .lx-slit[data-id=\"{id}\"]");
      let Some(slit) = crate::utils::query_selector::<web_sys::HtmlElement>(&selector) else {
        return;
      };
      let options = web_sys::ScrollIntoViewOptions::new();
      options.set_behavior(web_sys::ScrollBehavior::Smooth);
      options.set_block(web_sys::ScrollLogicalPosition::Nearest);
      slit.scroll_into_view_with_scroll_into_view_options(&options);
    });
  });

  Effect::new(move |_| {
    let handle = window_event_listener(leptos::ev::keydown, move |event: KeyboardEvent| {
      let typing = target_as::<HtmlInputElement>(&event).is_some();
      if event.key() == "/" && !typing {
        event.prevent_default();
        if let Some(input) = crate::utils::query_selector::<HtmlInputElement>(".lx-dock .bk-search")
        {
          let _ = input.focus();
        }
        return;
      }
      if event.key() == "Escape" {
        scrub.set(None);
        return;
      }
      if typing {
        return;
      }
      match event.key().as_str() {
        "ArrowRight" => step_plate(state, visible, locked, scrub, pending, 1),
        "ArrowLeft" => step_plate(state, visible, locked, scrub, pending, -1),
        "d" | "D" => {
          if let Some(image) = shown.get_untracked() {
            enqueue(state, image);
          }
        }
        "w" | "W" => {
          if let Some(image) = shown.get_untracked() {
            project(state, &image);
          }
        }
        "r" | "R" => state.refresh.update(|refresh| *refresh = !*refresh),
        _ => {}
      }
    });
    on_cleanup(move || handle.remove());
  });

  let geometry = move || {
    let narrow = narrow.get();
    let density = density.get();
    let hot = rail_hot.get();
    if narrow {
      let strip = match density {
        Density::Cinema => 0.0,
        Density::Tight => 76.0,
        Density::Gallery => 94.0,
      };
      // The strip sits between the plate and the dock, with one gap either side.
      let bottom = if strip == 0.0 {
        CONTENT_BOTTOM
      } else {
        CONTENT_BOTTOM + strip + GAP
      };
      return (CONTENT_TOP, GAP, bottom, strip, 0.0);
    }
    let rail = match (density, hot) {
      (Density::Tight, _) => 236.0,
      (Density::Cinema, true) => 148.0,
      (Density::Cinema, false) => 16.0,
      (Density::Gallery, true) => 156.0,
      (Density::Gallery, false) => 72.0,
    };
    // Both gaps count: one between the rail and the plate, one between the
    // rail and the stage edge.
    (CONTENT_TOP, rail + GAP * 2.0, CONTENT_BOTTOM, 0.0, rail)
  };

  let plate_style = move || {
    let (top, right, bottom, _, _) = geometry();
    format!("top:{top}px;right:{right}px;bottom:{bottom}px;")
  };
  let rail_style = move || {
    let narrow = narrow.get();
    let (_, _, _, strip, rail) = geometry();
    if narrow {
      let display = if strip == 0.0 { "none" } else { "flex" };
      format!(
        "left:{GAP}px;right:{GAP}px;bottom:{CONTENT_BOTTOM}px;height:{strip}px;width:auto;display:{display};"
      )
    } else {
      format!(
        "top:{CONTENT_TOP}px;right:{GAP}px;bottom:{CONTENT_BOTTOM}px;width:{rail}px;height:auto;display:grid;"
      )
    }
  };
  let shell_class = move || {
    let mut class = String::from("lx-stage");
    if narrow.get() {
      class.push_str(" lx-stage--narrow");
    }
    class.push_str(match density.get() {
      Density::Tight => " lx-density--tight",
      Density::Gallery => " lx-density--gallery",
      Density::Cinema => " lx-density--cinema",
    });
    class
  };

  let swipe_down = Rc::clone(&swipe);
  let swipe_up = Rc::clone(&swipe);

  view! {
    <div class=shell_class node_ref=root>
      <div class="lx-top">
        <div class="lx-mark">
          <strong>"KONA"</strong>
          <span>{move || if is_zh(state.i18n.get().language()) { "灯箱" } else { "LIGHTBOX" }}</span>
        </div>
        <div class="lx-sample" title=move || if is_zh(state.i18n.get().language()) { "从宽银幕取样的基色、主色、辅色" } else { "Base, primary, and ancillary sampled from the reel" }>
          <i class="lx-swatch lx-swatch--base"></i>
          <i class="lx-swatch lx-swatch--primary"></i>
          <i class="lx-swatch lx-swatch--ancillary"></i>
          <em>
            {move || match state.wallpaper.get() {
              Some((id, _)) => format!("#{id}"),
              None => {
                if is_zh(state.i18n.get().language()) { "壁纸".to_string() } else { "REEL".to_string() }
              }
            }}
          </em>
        </div>
      </div>

      <div
        class="lx-plate"
        style=plate_style
        on:wheel=move |event: WheelEvent| {
          let now = js_sys::Date::now();
          if now - wheel_at.get_untracked() < 160.0 {
            return;
          }
          wheel_at.set(now);
          event.prevent_default();
          let dir = if event.delta_y() > 0.0 { 1 } else { -1 };
          step_plate(state, visible, locked, scrub, pending, dir);
        }
        on:pointerdown=move |event: PointerEvent| {
          swipe_down.set(Some((event.client_x(), event.client_y())));
        }
        on:pointerup=move |event: PointerEvent| {
          let Some((x, y)) = swipe_up.take() else { return };
          if !narrow.get() {
            return;
          }
          let dx = event.client_x() - x;
          let dy = event.client_y() - y;
          if dx.abs() < 48 || dx.abs() < dy.abs() {
            return;
          }
          step_plate(state, visible, locked, scrub, pending, if dx < 0 { 1 } else { -1 });
        }
      >
        <div class="lx-hero">
          {move || under.get().map(|src| view! {
            <img class="lx-hero-under" src=src alt="" loading="eager" decoding="async"/>
          })}
          <For
            each=move || shown.get().map(|image| vec![image]).unwrap_or_default()
            key=|image| image.id
            children=move |image| {
              view! {
                <ImageTile
                  src=picture_url(&image)
                  alternative=image.preview.clone()
                  class_name="lx-hero-img"
                  priority=true
                />
              }
            }
          />
          // The first request has no plate image yet, so no tile spinner exists
          // to show for it: the plate would stay blank on the wallpaper until
          // the posts arrive.
          <Show when=move || shown.get().is_none() && state.loading.get()>
            <section class="bk-pending">
              <div class="loader"></div>
            </section>
          </Show>
        </div>
        <div class="lx-scan" aria-hidden="true"></div>
        <span class="lx-tick lx-tick--tl" aria-hidden="true"></span>
        <span class="lx-tick lx-tick--tr" aria-hidden="true"></span>
        <span class="lx-tick lx-tick--bl" aria-hidden="true"></span>
        <span class="lx-tick lx-tick--br" aria-hidden="true"></span>
        {move || shown.get().map(|image| {
          let queued = state.is_queued(&image.url);
          let zh = is_zh(state.i18n.get().language());
          let download_label = if queued {
            if zh { "已在队列" } else { "Queued" }
          } else if zh {
            "下载原图"
          } else {
            "Download"
          };
          let on_wall = state.wallpaper.get().is_some_and(|(id, _)| id == image.id);
          let wall_label = if on_wall {
            if zh { "正在放映" } else { "On the reel" }
          } else if zh {
            "送上宽银幕"
          } else {
            "Send to reel"
          };
          let for_download = Arc::clone(&image);
          let for_wall = Arc::clone(&image);
          view! {
            <div class="lx-actions">
              <button
                type="button"
                class="lx-btn lx-btn--primary"
                disabled=queued
                title=if zh { "下载原图 · D" } else { "Download · D" }
                on:click=move |_| enqueue(state, Arc::clone(&for_download))
              >
                <DownloadIcon/>
                <span class="lx-btn__label">{download_label}</span>
                <kbd class="lx-btn__key">"D"</kbd>
              </button>
              <button
                type="button"
                class="lx-btn"
                disabled=on_wall
                title=if zh { "送上宽银幕 · W" } else { "Send to reel · W" }
                on:click=move |_| project(state, &for_wall)
              >
                <ReelIcon/>
                <span class="lx-btn__label">{wall_label}</span>
                <kbd class="lx-btn__key">"W"</kbd>
              </button>
            </div>
          }
        })}
        <Show when=move || !state.loading.get() && visible.get().is_empty()>
          {move || {
            let zh = is_zh(state.i18n.get().language());
            let filtered = state.security.get() && state.tags.get().trim().is_empty();
            view! {
              <div class="lx-empty">
                <strong>"0"</strong>
                <p>{if filtered {
                  if zh { "这一页都在安全过滤之后" } else { "This page is entirely behind the safe filter" }
                } else if zh { "这些标签没有命中" } else { "Nothing matches these tags" }}</p>
                <button type="button" on:click=move |_| {
                  if state.security.get() && state.tags.get().trim().is_empty() {
                    state.security.set(false);
                  } else {
                    state.set_tags(String::new());
                  }
                }>
                  {if filtered { if zh { "查看全部" } else { "Show all" } } else if zh { "清空标签" } else { "Clear tags" }}
                </button>
              </div>
            }
          }}
        </Show>
      </div>

      <div
        class="lx-rail"
        style=rail_style
        on:mouseenter=move |_| rail_hot.set(true)
        on:mouseleave=move |_| {
          rail_hot.set(false);
          // Leaving the rail keeps whatever it last previewed: pressing on the
          // plate afterwards should continue from there, not from the image
          // that was on screen before the hover.
          if let Some(id) = scrub.get_untracked() {
            locked.set(Some(id));
          }
          scrub.set(None);
        }
      >
        <For
          each=move || visible.get()
          key=|image| (image.id, Arc::as_ptr(image) as usize)
          children=move |image| {
            let id = image.id;
            let url = image.url.clone();
            let on_plate = move || {
              scrub.get() == Some(id) || (scrub.get().is_none() && locked.get() == Some(id))
            };
            let class = move || {
              let mut class = String::from("lx-slit");
              if on_plate() { class.push_str(" lx-slit--on"); }
              if state.is_queued(&url) { class.push_str(" lx-slit--queued"); }
              class
            };
            view! {
              <button
                type="button"
                class=class
                data-id=id.to_string()
                aria-current=move || if on_plate() { "true" } else { "false" }
                title=format!("{}×{}  {}", image.width as u32, image.height as u32, image.tags.clone().unwrap_or_default())
                on:mouseenter=move |_| { if !narrow.get() { scrub.set(Some(id)); } }
                on:click=move |_| { locked.set(Some(id)); scrub.set(None); }
              >
                <ImageTile
                  src=image.preview.clone()
                  alternative="/image/error.png".to_string()
                  class_name="lx-slit-img"
                />
              </button>
            }
          }
        />
      </div>

      <div class="lx-dock">
        <div class="lx-dock__nav">
          <Nav/>
        </div>
        {move || shown.get().map(|image| {
          let tags = image.tags.clone().unwrap_or_default();
          let chips: Vec<String> = tags.split_whitespace().map(str::to_string).filter(|tag| !tag.is_empty()).collect();
          let zh = is_zh(state.i18n.get().language());
          view! {
            <div class="lx-dock__meta">
              <div class="lx-meta">
                <span class="lx-id" title=image.name.clone()>{image.id}</span>
                <span class="lx-dims">
                  {format!("{}×{}", image.width as u32, image.height as u32)}
                  <em>{format!(" · {}×{} · {}×{}", image.sample_width as u32, image.sample_height as u32, image.preview_width as u32, image.preview_height as u32)}</em>
                </span>
                <span class="lx-flag">{if image.security { if zh { "通过" } else { "Clear" } } else if zh { "敏感" } else { "Sensitive" }}</span>
                {chips.into_iter().map(|tag| {
                  let click = tag.clone();
                  view! {
                    <button type="button" class="lx-tag" on:click=move |event: web_sys::MouseEvent| toggle_tag(state, click.clone(), event.shift_key())>
                      {tag}
                    </button>
                  }
                }).collect_view()}
              </div>
            </div>
          }
        })}
        <div class="lx-dock__tools">
          <Search/>
          <DownloadList/>
          <Setting/>
          <span class="lx-dock__sep" aria-hidden="true"></span>
          <button
            type="button"
            class="lx-lang"
            title=move || if is_zh(state.i18n.get().language()) { "Switch to English" } else { "切换到中文" }
            on:click=move |_| {
              state.i18n.update(|i18n| {
                let next = match i18n.language() {
                  Language::Zh => Language::En,
                  Language::En => Language::Zh,
                };
                *i18n = crate::state::I18n::new(next);
              });
            }
          >
            {move || if is_zh(state.i18n.get().language()) { "EN" } else { "中文" }}
          </button>
          <div class="lx-density" role="group" aria-label="Rail density">
            <button
              type="button"
              class=move || if density.get() == Density::Tight { "is-on" } else { "" }
              aria-pressed=move || (density.get() == Density::Tight).to_string()
              on:click=move |_| density.set(Density::Tight)
            >
              {move || if is_zh(state.i18n.get().language()) { "密" } else { "Sheet" }}
            </button>
            <button
              type="button"
              class=move || if density.get() == Density::Gallery { "is-on" } else { "" }
              aria-pressed=move || (density.get() == Density::Gallery).to_string()
              on:click=move |_| density.set(Density::Gallery)
            >
              {move || if is_zh(state.i18n.get().language()) { "扫" } else { "Scan" }}
            </button>
            <button
              type="button"
              class=move || if density.get() == Density::Cinema { "is-on" } else { "" }
              aria-pressed=move || (density.get() == Density::Cinema).to_string()
              on:click=move |_| density.set(Density::Cinema)
            >
              {move || if is_zh(state.i18n.get().language()) { "阔" } else { "Wide" }}
            </button>
          </div>
          <span class="lx-keys">
            {move || if narrow.get() {
              if is_zh(state.i18n.get().language()) { "滑动切图 · 右下角下载" } else { "Swipe · download at the corner" }
            } else if is_zh(state.i18n.get().language()) {
              "滚轮或方向键切图，到头翻页"
            } else {
              "Wheel or arrows step the plate, then turn the page"
            }}
          </span>
        </div>
      </div>
    </div>
  }
}
