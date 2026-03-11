#![recursion_limit = "256"]

pub mod app;
#[cfg(feature = "ssr")]
pub mod common;
pub mod components;
pub mod error_template;
pub mod metadata;

// Server module contains server functions that need to be
// visible to both client and server.
// The #[server] macro handles conditional compilation
pub mod server;

pub mod table;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
    let _ = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .map(|el| el.class_list().remove_1("pre-hydration").ok());
}
