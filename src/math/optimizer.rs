use super::caffeine::{Dose, total_caffeine};
use super::constraints::{Constraints, valid_schedule};
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct OptimizationParams {
    pub dose_options: Vec<f64>, // allowed mg options
    pub step_minutes: i64,      // time granularity
}

pub struct UserSensitivity {
    half_life_hours: f64,
    sleep_decay_mg: f64,
    crash_threshold: f64,
}


fn score_schedule(doses: &[Dose], work_start: DateTime<Utc>, work_end: DateTime<Utc>) -> f64 {
    let mut score = 0.0;
    let mut t = work_start;
    let step = Duration::minutes(15);

    while t <= work_end {
        score += total_caffeine(doses, t);
        t += step;
    }

    score
}

pub fn optimize_schedule(
    doses_so_far: &[Dose],
    constraints: &Constraints,
    params: &OptimizationParams,
    work_start: DateTime<Utc>,
    work_end: DateTime<Utc>,
) -> Vec<Dose> {
    let mut best_schedule = doses_so_far.to_vec();
    let mut best_score = f64::MIN;

    let mut t = work_start;
    let step = Duration::minutes(params.step_minutes);

    while t <= work_end {
        for &mg in &params.dose_options {
            let mut candidate = doses_so_far.to_vec();
            candidate.push(Dose { mg, time: t });

            candidate.sort_by_key(|d| d.time);

            if !valid_schedule(&candidate, constraints) {
                continue;
            }

            let score = score_schedule(&candidate, work_start, work_end);

            if score > best_score {
                best_score = score;
                best_schedule = candidate;
            }
        }
        t += step;
    }

    best_schedule
}

fn test_optimizer() {
    let now = Utc::now();

    let constraints = Constraints {
        max_daily_mg: 400.0,
        min_gap_hours: 4.0,
        no_caffeine_after: now + Duration::hours(10),
    };

    let params = OptimizationParams {
        dose_options: vec![50.0, 100.0],
        step_minutes: 30,
    };

    let schedule = optimize_schedule(
        &[],
        &constraints,
        &params,
        now + Duration::hours(1),
        now + Duration::hours(8),
    );

    println!("Optimized schedule:");
    for d in schedule {
        println!("{:?}", d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::constraints::Constraints;
    use chrono::{Utc, Duration};

    #[test]
    fn test_optimizer_respects_constraints() {
        let now = Utc::now();

        let constraints = Constraints {
            max_daily_mg: 300.0,
            min_gap_hours: 4.0,
            no_caffeine_after: now + Duration::hours(12),
        };

        let params = OptimizationParams {
            dose_options: vec![50.0, 100.0],
            step_minutes: 60,
        };

        let schedule = optimize_schedule(
            &[],
            &constraints,
            &params,
            now + Duration::hours(1),
            now + Duration::hours(8),
        );

        // Check constraints
        assert!(valid_schedule(&schedule, &constraints));

        // Check total caffeine
        let total: f64 = schedule.iter().map(|d| d.mg).sum();
        assert!(total <= 300.0);

        println!("Optimized schedule:");
        for d in &schedule {
            println!("{:?}", d);
        }
    }
}