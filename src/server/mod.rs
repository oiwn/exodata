// Server functions module - compiled on both client and server
// #[server] macro handles conditional compilation of function bodies
pub mod functions;

// Server-only modules
#[cfg(feature = "ssr")]
pub mod common;

#[cfg(feature = "ssr")]
pub mod handlers;

#[cfg(test)]
mod tests;

#[cfg(feature = "ssr")]
use axum::Router;
#[cfg(feature = "ssr")]
pub use handlers::ApiState;

#[cfg(feature = "ssr")]
pub fn api_routes(state: ApiState) -> Router {
    handlers::api_routes(state)
}
