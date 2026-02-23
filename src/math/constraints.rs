use chrono::{Utc, DateTime};
use super::caffeine::{Dose};

pub struct Constraints {
    pub max_daily_mg: f64,
    pub min_gap_hours: f64,
    pub no_caffeine_after: DateTime<Utc>,
}

pub fn valid_schedule(schedule: &[Dose], constraints: &Constraints) -> bool {
    // max daily
    let total: f64 = schedule.iter().map(|d| d.mg).sum();
    if total > constraints.max_daily_mg {
        return false;
    }

    // minimum gap
    for pair in schedule.windows(2) {
        let dt = (pair[1].time - pair[0].time).num_minutes() as f64 / 60.0;
        if dt < constraints.min_gap_hours {
            return false;
        }
    }

    // bedtime cutoff
    for d in schedule {
        if d.time > constraints.no_caffeine_after {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, Duration};

    #[test]
    fn test_valid_schedule() {
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

        assert!(!valid_schedule(&schedule, &constraints));
    }
}