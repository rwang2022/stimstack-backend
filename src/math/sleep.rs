use chrono::{DateTime, Utc};
use crate::model::*;
use super::caffeine::{total_caffeine};

// Given a list of doses and a sleep time, predict a sleep quality score (0-100)
// based on how much caffeine is in the system at bedtime
pub fn predicted_sleep_score(doses: &[Dose], sleep_time: DateTime<Utc>, sensitivity: &UserSensitivity) -> f64 {
    let caffeine = total_caffeine(doses, sleep_time, sensitivity);
    // score is based on exponential decay of sleep quality as caffeine increases, scaled to 100
    let score = (-caffeine / sensitivity.sleep_decay_mg).exp() * 100.0;
    score.clamp(0.0, 100.0)
}
