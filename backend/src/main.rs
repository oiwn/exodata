use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/api", post(api_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Welcome to Axum"
}

async fn api_handler() -> &'static str {
    "This is the API endpoint"
}
