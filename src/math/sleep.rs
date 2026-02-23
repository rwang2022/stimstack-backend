use chrono::{DateTime, Utc};
use super::caffeine::{Dose, total_caffeine};

// Given a list of doses and a sleep time, predict a sleep quality score (0-100)
// based on how much caffeine is in the system at bedtime
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