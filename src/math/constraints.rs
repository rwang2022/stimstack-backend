struct Constraints {
    max_daily_mg: f64,
    min_gap_hours: f64,
    no_caffeine_after: DateTime<Utc>,
}

fn valid_schedule(schedule: &[Dose], constraints: &Constraints) -> bool {
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