use axum::{routing::post, Router, Json};
use stimstack_backend::math::optimizer::{OptimizerInput, OptimizerOutput};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/optimize", post(optimize_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn optimize_handler(Json(input): Json<OptimizerInput>) -> Json<OptimizerOutput> {
    let out = stimstack_backend::math::optimizer::optimize(input);
    Json(out)
}