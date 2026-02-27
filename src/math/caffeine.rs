use chrono::{DateTime, Utc, Duration};
use crate::model::*;

// helper function to model caffeine decay over time using an exponential decay function
fn caffeine_decay(c0: f64, hours: f64, sensitivity: &UserSensitivity) -> f64 {
    let k = (2.0_f64).ln() / sensitivity.half_life_hours; // decay constant based on half-life, usually 5hr
    c0 * (-k * hours).exp()
}

// Computes total caffeine in bloodstream at time t
pub fn total_caffeine(doses: &[Dose], t: DateTime<Utc>, sensitivity: &UserSensitivity) -> f64 {
    doses
        .iter()
        .map(|dose| {
            // for each dose, calculate how much caffeine remains at time t
            // if t is before the dose time, it contributes 0
            let dt = (t - dose.time).num_minutes() as f64 / 60.0;
            if dt < 0.0 {
                0.0
            } else {
                caffeine_decay(dose.mg, dt, sensitivity)
            }
        })
        .sum()
}

// Predict when caffeine will drop to 10mg (approximate crash point) based on current doses
pub fn predicted_crash(doses: &[Dose], now: DateTime<Utc>, sensitivity: &UserSensitivity) -> DateTime<Utc> {
    let current_caffeine = total_caffeine(doses, now, sensitivity);
    let threshold = sensitivity.crash_threshold; // mg level considered a crash
    if current_caffeine <= threshold {
        return now;
    }

    let k = (2.0_f64).ln() / sensitivity.half_life_hours; 
    let hours_until_crash = (current_caffeine / threshold).ln() / k; 
    let secs = (hours_until_crash * 3600.0).round() as i64;
    now + Duration::seconds(secs)
}

