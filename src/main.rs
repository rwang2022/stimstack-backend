use axum::{routing::get, Router};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
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
    
    let app = Router::new()
    .route("/health", get(health))
    .route("/timeline", post(timeline));
    
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "StimStack backend is alive (lightning)"
}

mod math;

use axum::{routing::post, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use math::caffeine::{Dose, total_caffeine};

#[derive(Deserialize)]
struct Intake {
    mg: f64,
    time: DateTime<Utc>,
}

#[derive(Deserialize)]
struct TimelineRequest {
    doses: Vec<Intake>,
}

#[derive(Serialize)]
struct TimelineResponse {
    total_caffeine: f64,
}

async fn timeline(Json(payload): Json<TimelineRequest>) -> Json<TimelineResponse> {
    let doses: Vec<Dose> = payload.doses.into_iter()
        .map(|d| Dose {mg: d.mg, time: d.time})
        .collect();

    let now = Utc::now();
    let total = total_caffeine(&doses, now);

    Json(TimelineResponse { 
        total_caffeine: total, 
    })
}
