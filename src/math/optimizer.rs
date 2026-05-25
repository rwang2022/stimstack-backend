use chrono::{DateTime, Utc, Duration};
use crate::model::constraints::Constraints;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dose {
    pub time: DateTime<Utc>,
    pub mg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerInput {
    pub half_life_hours: f64,
    pub constraints: Constraints,
    pub alertness_window: (DateTime<Utc>, DateTime<Utc>),
    pub sleep_time: DateTime<Utc>,
    pub dose_sizes: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerOutput {
    pub recommended_doses: Vec<Dose>,
    pub alertness_curve: Vec<(DateTime<Utc>, f64)>,
    pub sleep_score: f64,
    pub predicted_crash: Option<DateTime<Utc>>,
}

/// Maps total caffeine to an alertness score 0..100.
pub fn alertness_at(mg: f64) -> f64 {
    ((mg / 200.0) * 100.0).clamp(0.0, 100.0)
}

/// Generate all valid candidate schedules (combinations of 30-min slots) given dose sizes
/// and constraints. This is a brute-force grid generator; it prunes schedules that violate
/// `min_gap_hours` and `no_caffeine_after`.
pub fn generate_candidate_schedules(input: &OptimizerInput) -> Vec<Vec<Dose>> {
    let start = input.alertness_window.0;
    let end = input.alertness_window.1;
    let end = if input.constraints.no_caffeine_after < end {
        input.constraints.no_caffeine_after
    } else { end };

    // total slots every 30 minutes
    let mut slots = Vec::new();
    let mut t = start;
    while t <= end {
        slots.push(t);
        t = t + Duration::minutes(30);
    }
    let n = input.dose_sizes.len();
    let mut out = Vec::new();
    if n == 0 || slots.is_empty() { return out; }

    // quick constraint: total mg > max_daily_mg -> no candidates
    let total_mg: f64 = input.dose_sizes.iter().sum();
    if total_mg > input.constraints.max_daily_mg { return out; }

    // backtracking to pick indices into slots
    let min_gap = (input.constraints.min_gap_hours * 60.0) as i64; // minutes

    fn backtrack(slots: &Vec<DateTime<Utc>>, sizes: &Vec<f64>, idx: usize, cur: &mut Vec<DateTime<Utc>>, min_gap: i64, out: &mut Vec<Vec<DateTime<Utc>>>) {
        if idx == sizes.len() {
            out.push(cur.clone());
            return;
        }
        let start_pos = if idx == 0 { 0 } else {
            // ensure at least min_gap from previous
            let prev = cur[idx-1];
            // find first slot that is >= prev + min_gap
            let mut p = 0usize;
            while p < slots.len() && slots[p] < prev + Duration::minutes(min_gap) { p += 1; }
            p
        };
        for i in start_pos..slots.len() {
            // remaining slots must be enough for remaining doses
            if slots.len() - i < sizes.len() - idx { break; }
            cur.push(slots[i]);
            backtrack(slots, sizes, idx+1, cur, min_gap, out);
            cur.pop();
        }
    }

    let mut cur = Vec::new();
    let mut raw_out: Vec<Vec<DateTime<Utc>>> = Vec::new();
    backtrack(&slots, &input.dose_sizes, 0, &mut cur, min_gap, &mut raw_out);

    // convert to Dose vectors with mg mapping
    for schedule in raw_out {
        let mut doses = Vec::new();
        for (i, t) in schedule.into_iter().enumerate() {
            doses.push(Dose { time: t, mg: input.dose_sizes[i] });
        }
        out.push(doses);
    }

    out
}

/// Score a schedule. Returns `None` if any hard constraint is violated.
pub fn score_schedule(doses: &Vec<Dose>, constraints: &Constraints, alertness_window: (DateTime<Utc>, DateTime<Utc>), sleep_time: DateTime<Utc>, half_life_hours: f64) -> Option<f64> {
    // check total mg
    let total_mg: f64 = doses.iter().map(|d| d.mg).sum();
    if total_mg > constraints.max_daily_mg { return None; }
    // check no_caffeine_after and min_gap
    for d in doses {
        if d.time > constraints.no_caffeine_after { return None; }
    }
    for i in 1..doses.len() {
        let gap = doses[i].time.signed_duration_since(doses[i-1].time).num_minutes();
        if gap < (constraints.min_gap_hours * 60.0) as i64 { return None; }
    }

    // compute average alertness during window sampled every 30 minutes
    let mut t = alertness_window.0;
    let mut samples = 0u32;
    let mut sum_alert = 0.0f64;
    let pairs: Vec<(DateTime<Utc>, f64)> = doses.iter().map(|d| (d.time, d.mg)).collect();
    while t <= alertness_window.1 {
        let mg = crate::math::caffeine::total_caffeine(&pairs, t, half_life_hours);
        sum_alert += alertness_at(mg);
        samples += 1;
        t = t + Duration::minutes(30);
    }
    if samples == 0 { return None; }
    let avg_alert = sum_alert / (samples as f64);

    let sleep_score = crate::math::sleep::predicted_sleep_score(&pairs, sleep_time, half_life_hours);

    let score = 0.7 * avg_alert + 0.3 * sleep_score;
    Some(score)
}

pub fn optimize(input: OptimizerInput) -> OptimizerOutput {
    let candidates = generate_candidate_schedules(&input);
    let mut best_score = None::<f64>;
    let mut best_schedule: Option<Vec<Dose>> = None;
    for cand in candidates {
        if let Some(s) = score_schedule(&cand, &input.constraints, input.alertness_window, input.sleep_time, input.half_life_hours) {
            if best_score.is_none() || s > best_score.unwrap() {
                best_score = Some(s);
                best_schedule = Some(cand);
            }
        }
    }
    // fallback: empty
    let chosen = best_schedule.unwrap_or_else(|| generate_candidate_schedule_simple(&input));

    // build alertness curve
    let mut curve = Vec::new();
    let mut t = input.alertness_window.0;
    let pairs: Vec<(DateTime<Utc>, f64)> = chosen.iter().map(|d| (d.time, d.mg)).collect();
    while t <= input.alertness_window.1 {
        let mg = crate::math::caffeine::total_caffeine(&pairs, t, input.half_life_hours);
        curve.push((t, alertness_at(mg)));
        t = t + Duration::minutes(30);
    }
    let sleep_score = crate::math::sleep::predicted_sleep_score(&pairs, input.sleep_time, input.half_life_hours);
    let predicted_crash = crate::math::caffeine::predicted_crash(&pairs, input.alertness_window.0, input.half_life_hours, 20.0);

    OptimizerOutput {
        recommended_doses: chosen,
        alertness_curve: curve,
        sleep_score,
        predicted_crash,
    }
}

// Simple fallback generator used when no candidates passed scoring: evenly spaced inside window
fn generate_candidate_schedule_simple(input: &OptimizerInput) -> Vec<Dose> {
    let mut out = Vec::new();
    let start = input.alertness_window.0;
    let end = input.alertness_window.1;
    let n = input.dose_sizes.len() as i64;
    if n == 0 { return out; }
    let total_minutes = (end.signed_duration_since(start)).num_minutes();
    let step = total_minutes / n;
    for (i, mg) in input.dose_sizes.iter().enumerate() {
        let dt = start + Duration::minutes(step * (i as i64));
        out.push(Dose { time: dt, mg: *mg });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, TimeZone};

    #[test]
    fn test_total_caffeine_decay() {
        use crate::math::caffeine::total_caffeine;
        let t0 = Utc.ymd(2026,5,25).and_hms(8,0,0);
        let doses = vec![(t0, 100.0)];
        let mg_now = total_caffeine(&doses, t0, 5.0);
        assert!((mg_now - 100.0).abs() < 1e-6);
        let t5 = t0 + Duration::hours(5);
        let mg_after = total_caffeine(&doses, t5, 5.0);
        assert!((mg_after - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_optimizer_basic() {
        use chrono::TimeZone;
        let start = Utc.ymd(2026,5,25).and_hms(9,0,0);
        let end = Utc.ymd(2026,5,25).and_hms(17,0,0);
        let sleep = Utc.ymd(2026,5,25).and_hms(23,0,0);
        let constraints = Constraints { max_daily_mg: 400.0, min_gap_hours: 4.0, no_caffeine_after: Utc.ymd(2026,5,25).and_hms(20,0,0) };
        let input = OptimizerInput {
            half_life_hours: 5.0,
            constraints: constraints.clone(),
            alertness_window: (start, end),
            sleep_time: sleep,
            dose_sizes: vec![95.0, 95.0],
        };
        let out = optimize(input);
        assert_eq!(out.recommended_doses.len(), 2);
        assert!(out.sleep_score >= 0.0 && out.sleep_score <= 100.0);
    }
}
