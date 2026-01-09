//! {{SERVICE_NAME}} - Leptos SSR Application
//!
//! Architecture:
//! - `app/` - Leptos components and pages (isomorphic)
//! - `server/` - Actix-web server, actors, handlers (SSR only)
//! - `domain/` - Business logic, models, services (shared)
//! - `infra/` - Database, cache adapters (SSR only)

pub mod app;
pub mod domain;

#[cfg(feature = "ssr")]
pub mod server;

#[cfg(feature = "ssr")]
pub mod infra;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
