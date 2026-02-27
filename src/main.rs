// use tracing_subscriber;
use axum::{routing::post, Router, Json};
use chrono::{Utc, DateTime, Duration};
use serde::{Deserialize, Serialize};

// Module declarations - organize your app into logical sections
pub mod math;      // Caffeine calculations
pub mod model;     // Data structures

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
    Json(TimelineResponse { 
        total_caffeine: 0.0,
        crash_time: Utc::now(),
        sleep_score: 0.0,
    })
}