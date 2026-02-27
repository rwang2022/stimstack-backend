// use tracing_subscriber;
use axum::{routing::post, Router, Json};
use chrono::{Utc, DateTime, Duration};
use serde::{Deserialize, Serialize};

// Module declarations - organize your app into logical sections
pub mod math;      // Caffeine calculations
pub mod model;     // Data structures
pub mod engine;    // Constraint validation

// Import what you need explicitly (easier to track dependencies)
use math::*;
use model::Dose;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .unwrap();
    
    let app = Router::new()
        .route("/timeline", post(timeline));
    
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct TimelineRequest {
    doses: Vec<Dose>, // list of caffeine intakes
}

#[derive(Serialize)]
struct TimelineResponse {
    total_caffeine: f64, // how much caffeine is currently in the bloodstream
    crash_time: DateTime<Utc>, // when caffeine will drop to crash level (10mg)
    sleep_score: f64, // predicted sleep quality score (0-100) based on caffeine at bedtime
}

async fn timeline(Json(payload): Json<TimelineRequest>) -> Json<TimelineResponse> {
    let doses: Vec<Dose> = payload.doses;

    let now = Utc::now();
    let total = total_caffeine(&doses, now);
    
    print!("your total caffeine rn is {:.2} mg", total);
    print!("your predicted crash is at {}", predicted_crash(&doses, now));
    print!("your sleep score if you slept now is {:.2}", sleep_score(&doses, now));

    Json(TimelineResponse { 
        total_caffeine: total, 
        crash_time: predicted_crash(&doses, now),
        sleep_score: sleep_score(&doses, now),
    })
}