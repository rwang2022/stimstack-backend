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


    use chrono::Utc;
    use math::caffeine::{Dose, total_caffeine};

    let doses = vec![
        Dose { mg: 200.0, time: Utc::now() - chrono::Duration::hours(2) }, 
        Dose { mg: 160.0, time: Utc::now() - chrono::Duration::hours(6) }, 
    ];

    let current = total_caffeine(&doses, Utc::now());
    println!("Current caffeine level: {:.2} mg", current);
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "StimStack backend is alive (lightning)"
}

mod math;
