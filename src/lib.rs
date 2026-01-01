pub mod app;
#[cfg(feature = "ssr")]
pub mod common;
pub mod components;
pub mod error_template;

// Server module contains server functions that need to be visible to both client and server
// The #[server] macro handles conditional compilation
pub mod server;

pub mod table;

#[cfg(feature = "ssr")]
pub mod stellarhosts;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
