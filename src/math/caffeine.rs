use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dose {
    pub mg: f64, 
    pub time: DateTime<Utc>,
}

// Computes caffeine decay using half-life model
pub fn caffeine_decay(c0: f64, hours: f64) -> f64 {
    let k = (2.0_f64).ln() / 5.0; // 5 hour half-life model
    c0 * (-k * hours).exp() // c(t) = c0 * e^(-kt) where k = ln(2) / half_life (5 hours)
}

// Computes total caffeine in bloodstream at time t
pub fn total_caffeine(doses: &[Dose], t: DateTime<Utc>) -> f64 {
    doses
        .iter()
        .map(|dose| {
            // for each dose, calculate how much caffeine remains at time t 
            // if t is before the dose time, it contributes 0 
            let dt = (t - dose.time).num_minutes() as f64 / 60.0;
            if dt < 0.0 {
                0.0
            } else {
                caffeine_decay(dose.mg, dt)
            }
        })
        .sum()
}

// Predict when caffeine will drop to 10mg (approximate crash point) based on current doses
pub fn predicted_crash(doses: &[Dose], now: DateTime<Utc>) -> DateTime<Utc> {
    let current_caffeine = total_caffeine(doses, now);
    let k = (2.0_f64).ln() / 5.0; // 5 hour half-life model
    let threshold = 10.0; // mg level considered a crash
    let hours_until_crash = (current_caffeine / threshold).ln() / k; // when caffeine decays to 10mg
    now + Duration::milliseconds((hours_until_crash * 3600.0 * 1000.0) as i64)
}