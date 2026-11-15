//! Inline SVG icons.
//!
//! The paths are taken verbatim from Font Awesome Free 6, Lucide and
//! Inlining them removes three icon-pack dependencies from the bundle.

use leptos::prelude::*;

/// Font Awesome "download" (solid) icon.
#[component]
pub fn DownloadIcon(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
  view! {
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 512 512"
      width="1em"
      height="1em"
      fill="currentColor"
      class=class
    >
      <path d="M288 32c0-17.7-14.3-32-32-32s-32 14.3-32 32l0 242.7-73.4-73.4c-12.5-12.5-32.8-12.5-45.3 0s-12.5 32.8 0 45.3l128 128c12.5 12.5 32.8 12.5 45.3 0l128-128c12.5-12.5 12.5-32.8 0-45.3s-32.8-12.5-45.3 0L288 274.7 288 32zM64 352c-35.3 0-64 28.7-64 64l0 32c0 35.3 28.7 64 64 64l384 0c35.3 0 64-28.7 64-64l0-32c0-35.3-28.7-64-64-64l-101.5 0-45.3 45.3c-25 25-65.5 25-90.5 0L165.5 352 64 352zm368 56a24 24 0 1 1 0 48 24 24 0 1 1 0-48z"/>
    </svg>
  }
}

/// Lucide "monitor" icon, used to send the plate to the wallpaper.
#[component]
pub fn ReelIcon(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
  view! {
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      width="1em"
      height="1em"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      class=class
    >
      <rect width="20" height="14" x="2" y="3" rx="1"/>
      <line x1="8" x2="16" y1="21" y2="21"/>
      <line x1="12" x2="12" y1="17" y2="21"/>
    </svg>
  }
}

/// Lucide "refresh-ccw" icon, used to retry a failed download.
#[component]
pub fn RefreshIcon(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
  view! {
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      width="1em"
      height="1em"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      class=class
    >
      <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
      <path d="M3 3v5h5"/>
      <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
      <path d="M16 16h5v5"/>
    </svg>
  }
}

/// Bootstrap "check-circle-fill" icon.
#[component]
pub fn CheckCircleIcon(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
  view! {
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 16 16"
      width="16px"
      height="16px"
      fill="currentColor"
      class=class
    >
      <path d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0m-3.97-3.03a.75.75 0 0 0-1.08.022L7.477 9.417 5.384 7.323a.75.75 0 0 0-1.06 1.06L6.97 11.03a.75.75 0 0 0 1.079-.02l3.992-4.99a.75.75 0 0 0-.01-1.05z"/>
    </svg>
  }
}

/// Bootstrap "info-circle-fill" icon.
#[component]
pub fn InfoCircleIcon(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
  view! {
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 16 16"
      width="16px"
      height="16px"
      fill="currentColor"
      class=class
    >
      <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16m.93-9.412-1 4.705c-.07.34.029.533.304.533.194 0 .487-.07.686-.246l-.088.416c-.287.346-.92.598-1.465.598-.703 0-1.002-.422-.808-1.319l.738-3.468c.064-.293.006-.399-.287-.47l-.451-.081.082-.381 2.29-.287zM8 5.5a1 1 0 1 1 0-2 1 1 0 0 1 0 2"/>
    </svg>
  }
}
