use chrono::{DateTime, Utc};
use crate::math::caffeine::total_caffeine;

/// Simple sleep score: 100 is perfect sleep, losses are proportional to
/// remaining caffeine at sleep time and user sensitivity (half-life).
pub fn predicted_sleep_score(doses: &[(DateTime<Utc>, f64)], sleep_time: DateTime<Utc>, half_life_hours: f64) -> f64 {
    let mg = total_caffeine(doses, sleep_time, half_life_hours);
    // heuristic: each 50mg remaining reduces score by 10 points
    let penalty = (mg / 50.0) * 10.0;
    (100.0 - penalty).clamp(0.0, 100.0)
}
