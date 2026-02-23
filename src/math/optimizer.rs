use chrono::{Utc, DateTime, Duration};
use super::caffeine::{Dose, total_caffeine};




pub struct UserSensitivity {
    half_life_hours: f64,
    sleep_decay_mg: f64,
    crash_threshold: f64,
}
