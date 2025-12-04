pub mod handlers;

#[cfg(test)]
mod tests;

use axum::Router;

pub fn api_routes() -> Router {
    handlers::api_routes()
}