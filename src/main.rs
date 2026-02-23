use tracing_subscriber;
use axum::{routing::post, Router, Json};

mod math;
use chrono::{Utc, DateTime, Duration};
use serde::{Deserialize, Serialize};
use math::caffeine::{Dose, total_caffeine, predicted_crash};
use math::sleep::sleep_score;
use math::constraints::{Constraints, valid_schedule};

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

fn test_constraints() {
    let schedule = vec![
        Dose { mg: 100.0, time: Utc::now() - Duration::hours(1) },
        Dose { mg: 150.0, time: Utc::now() },
        Dose { mg: 200.0, time: Utc::now() + Duration::hours(1) },
    ];

    let constraints = Constraints {
        max_daily_mg: 400.0,
        min_gap_hours: 2.0,
        no_caffeine_after: Utc::now() + Duration::hours(12),
    };

    println!("Schedule valid? {}", valid_schedule(&schedule, &constraints)); // should be false due to min gap and max daily
    assert!(!valid_schedule(&schedule, &constraints)); // exceeds max daily
} 