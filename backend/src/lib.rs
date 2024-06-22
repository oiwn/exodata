use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio;

pub fn run_server() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    runtime.block_on(async {
        let app = Router::new()
            .route("/", get(root))
            .route("/api", get(api_handler));

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        println!("Server running at http://{}", addr);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}

async fn root() -> &'static str {
    "Welcome to Axum"
}

async fn api_handler() -> &'static str {
    "This is the API endpoint"
}
