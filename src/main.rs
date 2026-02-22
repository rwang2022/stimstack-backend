use axum::{routing::get, Router};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .unwrap();

    println!("Server running on http://localhost:8000");

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "StimStack backend is alive (lightning)"
}