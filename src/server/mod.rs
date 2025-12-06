pub mod handlers;
pub mod functions;

#[cfg(test)]
mod tests;

use axum::Router;
pub use handlers::ApiState;

pub fn api_routes(state: ApiState) -> Router {
    handlers::api_routes(state)
}