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

// Predict when caffeine will drop to (user-based crash point) based on current doses
// Returns the predicted crash time as some time in the future from now
pub fn predicted_crash(doses: &[Dose], sensitivity: &UserSensitivity) -> DateTime<Utc> {
    let now = Utc::now();

    let current_caffeine = total_caffeine(doses, now, sensitivity);
    let threshold = sensitivity.crash_threshold; // user-specific crash threshold
    if current_caffeine <= threshold {
        return now;
    }

    let k = (2.0_f64).ln() / sensitivity.half_life_hours; 
    let hours_until_crash = (current_caffeine / threshold).ln() / k; 
    let secs = (hours_until_crash * 3600.0).round() as i64;

    now + Duration::seconds(secs) // returns the predicted crash time, as some time in the future from now
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn test_sensitivity() -> UserSensitivity {
        UserSensitivity {
            half_life_hours: 5.0,
            sleep_decay_mg: 50.0,
            crash_threshold: 10.0,
        }
    }

    #[test]
    fn test_caffeine_decay_at_half_life() {
        let sens = test_sensitivity();
        let decayed = caffeine_decay(100.0, 5.0, &sens);
        assert!((decayed - 50.0).abs() < 0.1); // should be ~50mg after 5 hours
    }

    #[test]
    fn test_total_caffeine_single_dose() {
        let sens = test_sensitivity();
        let now = Utc.with_ymd_and_hms(2026, 2, 27, 14, 0, 0).unwrap();
        let dose_time = Utc.with_ymd_and_hms(2026, 2, 27, 9, 0, 0).unwrap();
        let doses = vec![Dose {
            mg: 100.0,
            time: dose_time,
        }];

        let total = total_caffeine(&doses, now, &sens);
        // 5 hours have passed, so 100 * 0.5 = 50mg
        assert!((total - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_total_caffeine_multiple_doses() {
        let sens = test_sensitivity();
        let now = Utc.with_ymd_and_hms(2026, 2, 27, 14, 0, 0).unwrap();
        let doses = vec![
            Dose {
                mg: 100.0,
                time: Utc.with_ymd_and_hms(2026, 2, 27, 9, 0, 0).unwrap(),
            }, // 5 hours ago: 50mg
            Dose {
                mg: 200.0,
                time: Utc.with_ymd_and_hms(2026, 2, 27, 13, 0, 0).unwrap(),
            }, // 1 hour ago: 200 * 0.5^(1/5) ≈ 171mg
        ];

        let total = total_caffeine(&doses, now, &sens);
        assert!(total > 200.0 && total < 230.0); // rough range
    }

    #[test]
    fn test_predicted_crash_in_future() {
        let sens = test_sensitivity();
        let now = Utc.with_ymd_and_hms(2026, 2, 27, 9, 0, 0).unwrap();
        let doses = vec![Dose {
            mg: 200.0,
            time: now,
        }];

        let crash = predicted_crash(&doses, &sens);
        assert!(crash > now); // crash is in the future
        assert!(crash.signed_duration_since(now).num_hours() > 0);
        assert!(crash.signed_duration_since(now).num_hours() < 24); // reasonable bound
    }

    #[test]
    fn test_no_doses() {
        let sens = test_sensitivity();
        let now = Utc.with_ymd_and_hms(2026, 2, 27, 14, 0, 0).unwrap();
        let doses: Vec<Dose> = vec![];

        let total = total_caffeine(&doses, now, &sens);
        assert_eq!(total, 0.0);
    }
}