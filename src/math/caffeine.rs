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

// Given a list of doses and a sleep time, predict a sleep quality score (0-100) based on how much caffeine is in the system at bedtime
pub fn sleep_score(doses: &[Dose], sleep_time: DateTime<Utc>) -> f64 {
    let caffeine_at_sleep = total_caffeine(doses, sleep_time);
    if caffeine_at_sleep <= 10.0 {
        100.0 // no significant caffeine, perfect sleep score
    } else {
        // simple linear penalty: every 10mg above threshold reduces score by 20 points
        // going to sleep with 120mg (~a can of monster) reduces score to 0 
        let excess = caffeine_at_sleep - 10.0;
        let penalty = (excess / 10.0).ceil() * 20.0;
        (100.0 - penalty).max(0.0) // score can't go below 0
    }
}
