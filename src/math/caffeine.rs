use chrono::{DateTime, Utc, Duration};

/// Simple exponential decay model for caffeine in mg.
/// `doses` is a slice of (time, mg) pairs.
pub fn total_caffeine(doses: &[(DateTime<Utc>, f64)], t: DateTime<Utc>, half_life_hours: f64) -> f64 {
    let mut sum = 0.0;
    for (td, mg) in doses {
        if *td <= t {
            let secs = (t.signed_duration_since(*td)).num_seconds() as f64;
            let hours = secs / 3600.0;
            let decay = 2f64.powf(-hours / half_life_hours);
            sum += mg * decay;
        }
    }
    sum
}

/// Predicts a naive crash time: the first time after now when total caffeine
/// falls below `threshold_mg`. This is a placeholder; optimizer will use
/// more sophisticated scoring.
pub fn predicted_crash(doses: &[(DateTime<Utc>, f64)], now: DateTime<Utc>, half_life_hours: f64, threshold_mg: f64) -> Option<DateTime<Utc>> {
    // search forward in 30-minute steps up to 48 hours
    for i in 0..(48 * 2) {
        let t = now + Duration::minutes(30 * i);
        if total_caffeine(doses, t, half_life_hours) <= threshold_mg {
            return Some(t);
        }
    }
    None
}
