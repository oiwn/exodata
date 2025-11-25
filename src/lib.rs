pub mod app;
#[cfg(feature = "ssr")]
pub mod common;
pub mod error_template;
#[cfg(feature = "ssr")]
#[cfg(feature = "ssr")]
pub mod stellarhosts;
pub mod tables;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
