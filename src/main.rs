// use tracing_subscriber;
use axum::{routing::post, Router, Json};
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};

// Module declarations - organize your app into logical sections
pub mod math;      // Caffeine calculations
pub mod model;     // Data structures

// Import what you need explicitly (easier to track dependencies)
use model::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .unwrap();
    
    let app = Router::new()
        .route("/timeline", post(calculator));
    
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct CalculatorInputs {
    doses: Vec<Dose>, // list of caffeine intakes
    sleep_time: DateTime<Utc>, // when user plans to sleep
    profile: UserProfile, // user-specific factors affecting caffeine metabolism
}

#[derive(Serialize)]
struct CalculatorOutputs {
    total_caffeine: f64, // how much caffeine is currently in the bloodstream
    predicted_crash: DateTime<Utc>, // when caffeine will drop to crash level (10mg)
    sleep_score: f64, // predicted sleep quality score (0-100) based on caffeine at bedtime
}

async fn calculator(Json(payload): Json<CalculatorInputs>) -> Json<CalculatorOutputs> {
    let sensitivity = UserSensitivity::from_profile(&payload.profile);

    let total_caffeine = math::caffeine::total_caffeine(&payload.doses, Utc::now(), &sensitivity);
    let predicted_crash = math::caffeine::predicted_crash(&payload.doses, Utc::now(), &sensitivity);
    let predicted_sleep_score = math::sleep::predicted_sleep_score(&payload.doses, payload.sleep_time, &sensitivity);

    Json(CalculatorOutputs { 
        total_caffeine,
        predicted_crash,
        sleep_score: predicted_sleep_score,
    })
}