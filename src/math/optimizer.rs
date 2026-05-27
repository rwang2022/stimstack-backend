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

/// Maps total caffeine mg to an alertness score 0..100.
pub fn alertness_at(mg: f64) -> f64 {
    ((mg / 200.0) * 100.0).clamp(0.0, 100.0)
}

/// Generate all valid candidate schedules (combinations of 30-min slots) given dose sizes
/// and constraints. Schedules respect `min_gap_hours` and `no_caffeine_after`.
pub fn generate_candidate_schedules(input: &OptimizerInput) -> Vec<Vec<Dose>> {
    let total_mg: f64 = input.dose_sizes.iter().sum();
    if input.dose_sizes.is_empty() || total_mg > input.constraints.max_daily_mg {
        return Vec::new();
    }

    let start = input.alertness_window.0;
    let cutoff = input.constraints.no_caffeine_after.min(input.alertness_window.1);
    let slot_count = ((cutoff - start).num_minutes() / 30 + 1).max(0) as usize;
    let slots: Vec<DateTime<Utc>> = (0..slot_count)
        .map(|i| start + Duration::minutes(30 * i as i64))
        .collect();

    let min_gap_minutes = (input.constraints.min_gap_hours * 60.0) as i64;
    let mut schedules = Vec::new();
    let mut cur = Vec::with_capacity(input.dose_sizes.len());
    backtrack(&slots, &input.dose_sizes, 0, &mut cur, min_gap_minutes, &mut schedules);
    schedules
}

fn backtrack(
    slots: &[DateTime<Utc>],
    sizes: &[f64],
    idx: usize,
    cur: &mut Vec<Dose>,
    min_gap_minutes: i64,
    out: &mut Vec<Vec<Dose>>,
) {
    if idx == sizes.len() {
        out.push(cur.clone());
        return;
    }
    let start_pos = if idx == 0 {
        0
    } else {
        let earliest = cur[idx - 1].time + Duration::minutes(min_gap_minutes);
        slots.partition_point(|&s| s < earliest)
    };
    for i in start_pos..slots.len() {
        if slots.len() - i < sizes.len() - idx { break; }
        cur.push(Dose { time: slots[i], mg: sizes[idx] });
        backtrack(slots, sizes, idx + 1, cur, min_gap_minutes, out);
        cur.pop();
    }
}

/// Score a candidate schedule. Returns `None` if any hard constraint is violated.
/// For internal use in `optimize`, prefer `compute_score` directly (constraints
/// are already guaranteed by `generate_candidate_schedules`).
pub fn score_schedule(
    doses: &[Dose],
    constraints: &Constraints,
    alertness_window: (DateTime<Utc>, DateTime<Utc>),
    sleep_time: DateTime<Utc>,
    half_life_hours: f64,
) -> Option<f64> {
    let total_mg: f64 = doses.iter().map(|d| d.mg).sum();
    if total_mg > constraints.max_daily_mg { return None; }
    for d in doses {
        if d.time > constraints.no_caffeine_after { return None; }
    }
    for w in doses.windows(2) {
        let gap = w[1].time.signed_duration_since(w[0].time).num_minutes();
        if gap < (constraints.min_gap_hours * 60.0) as i64 { return None; }
    }
    Some(compute_score(doses, alertness_window, sleep_time, half_life_hours))
}

/// Compute the alertness + sleep composite score for a schedule.
/// No constraint validation — assumes the caller has already verified constraints.
fn compute_score(
    doses: &[Dose],
    alertness_window: (DateTime<Utc>, DateTime<Utc>),
    sleep_time: DateTime<Utc>,
    half_life_hours: f64,
) -> f64 {
    let pairs: Vec<(DateTime<Utc>, f64)> = doses.iter().map(|d| (d.time, d.mg)).collect();

    let mut t = alertness_window.0;
    let mut sum_alert = 0.0f64;
    let mut samples = 0u32;
    while t <= alertness_window.1 {
        sum_alert += alertness_at(crate::math::caffeine::total_caffeine(&pairs, t, half_life_hours));
        samples += 1;
        t += Duration::minutes(30);
    }
    let avg_alert = if samples > 0 { sum_alert / samples as f64 } else { 0.0 };
    let sleep_score = crate::math::sleep::predicted_sleep_score(&pairs, sleep_time, half_life_hours);

    0.7 * avg_alert + 0.3 * sleep_score
}

pub fn optimize(input: OptimizerInput) -> OptimizerOutput {
    let chosen = generate_candidate_schedules(&input)
        .into_iter()
        .map(|c| {
            let s = compute_score(&c, input.alertness_window, input.sleep_time, input.half_life_hours);
            (c, s)
        })
        .max_by(|(_, sa), (_, sb)| sa.partial_cmp(sb).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(c, _)| c)
        .unwrap_or_else(|| evenly_spaced_fallback(&input));

    let pairs: Vec<(DateTime<Utc>, f64)> = chosen.iter().map(|d| (d.time, d.mg)).collect();

    let mut curve = Vec::new();
    let mut t = input.alertness_window.0;
    while t <= input.alertness_window.1 {
        let mg = crate::math::caffeine::total_caffeine(&pairs, t, input.half_life_hours);
        curve.push((t, alertness_at(mg)));
        t += Duration::minutes(30);
    }

    OptimizerOutput {
        recommended_doses: chosen,
        alertness_curve: curve,
        sleep_score: crate::math::sleep::predicted_sleep_score(&pairs, input.sleep_time, input.half_life_hours),
        predicted_crash: crate::math::caffeine::predicted_crash(&pairs, input.alertness_window.0, input.half_life_hours, 20.0),
    }
}

/// Fallback: evenly-spaced doses inside the alertness window.
/// Used when no candidates satisfy the constraints (e.g. window too narrow).
fn evenly_spaced_fallback(input: &OptimizerInput) -> Vec<Dose> {
    let n = input.dose_sizes.len() as i64;
    if n == 0 { return Vec::new(); }
    let total_minutes = (input.alertness_window.1 - input.alertness_window.0).num_minutes();
    let step = total_minutes / n;
    input.dose_sizes.iter().enumerate()
        .map(|(i, &mg)| Dose {
            time: input.alertness_window.0 + Duration::minutes(step * i as i64),
            mg,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, TimeZone};

    fn base_constraints() -> Constraints {
        Constraints {
            max_daily_mg: 400.0,
            min_gap_hours: 4.0,
            no_caffeine_after: Utc.with_ymd_and_hms(2026, 5, 25, 20, 0, 0).unwrap(),
        }
    }

    fn base_input(dose_sizes: Vec<f64>) -> OptimizerInput {
        OptimizerInput {
            half_life_hours: 5.0,
            constraints: base_constraints(),
            alertness_window: (
                Utc.with_ymd_and_hms(2026, 5, 25, 9, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 5, 25, 17, 0, 0).unwrap(),
            ),
            sleep_time: Utc.with_ymd_and_hms(2026, 5, 25, 23, 0, 0).unwrap(),
            dose_sizes,
        }
    }

    // ── caffeine decay ──────────────────────────────────────────────────────

    #[test]
    fn test_total_caffeine_decay() {
        use crate::math::caffeine::total_caffeine;
        let t0 = Utc.with_ymd_and_hms(2026, 5, 25, 8, 0, 0).unwrap();
        let doses = vec![(t0, 100.0)];
        assert!((total_caffeine(&doses, t0, 5.0) - 100.0).abs() < 1e-6);
        assert!((total_caffeine(&doses, t0 + Duration::hours(5), 5.0) - 50.0).abs() < 1e-6);
    }

    // ── alertness scoring ───────────────────────────────────────────────────

    #[test]
    fn test_alertness_at_bounds() {
        assert!((alertness_at(0.0)   - 0.0).abs()   < 1e-6);
        assert!((alertness_at(100.0) - 50.0).abs()  < 1e-6);
        assert!((alertness_at(200.0) - 100.0).abs() < 1e-6);
        assert!((alertness_at(400.0) - 100.0).abs() < 1e-6); // clamped at 100
    }

    // ── sleep scoring ───────────────────────────────────────────────────────

    #[test]
    fn test_sleep_score_no_caffeine() {
        use crate::math::sleep::predicted_sleep_score;
        let sleep = Utc.with_ymd_and_hms(2026, 5, 25, 23, 0, 0).unwrap();
        assert!((predicted_sleep_score(&[], sleep, 5.0) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_sleep_score_high_caffeine() {
        use crate::math::sleep::predicted_sleep_score;
        // 500 mg dose taken 1 minute before sleep → almost all still in system
        let t0 = Utc.with_ymd_and_hms(2026, 5, 25, 22, 59, 0).unwrap();
        let sleep = Utc.with_ymd_and_hms(2026, 5, 25, 23, 0, 0).unwrap();
        
        let doses = vec![(t0, 500.0)];
        let score = predicted_sleep_score(&doses, sleep, 5.0);

        assert!(score < 10.0, "expected near-zero sleep score, got {score}");
    }

    // ── candidate generation ────────────────────────────────────────────────

    #[test]
    fn test_generate_candidates_over_mg_limit() {
        // 300 + 200 = 500 mg > 400 mg limit
        let candidates = generate_candidate_schedules(&base_input(vec![300.0, 200.0]));
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_generate_candidates_min_gap_respected() {
        let input = base_input(vec![95.0, 95.0]);
        let candidates = generate_candidate_schedules(&input);
        assert!(!candidates.is_empty(), "expected at least one candidate");
        for schedule in &candidates {
            for w in schedule.windows(2) {
                let gap = w[1].time.signed_duration_since(w[0].time).num_minutes();
                assert!(gap >= 240, "min_gap_hours=4.0 violated: gap was {gap} min");
            }
        }
    }

    #[test]
    fn test_generate_candidates_no_caffeine_after_respected() {
        let input = base_input(vec![95.0, 95.0]);
        let cutoff = input.constraints.no_caffeine_after;
        for schedule in generate_candidate_schedules(&input) {
            for dose in schedule {
                assert!(dose.time <= cutoff, "dose scheduled after no_caffeine_after");
            }
        }
    }

    // ── score_schedule (public API) ─────────────────────────────────────────

    #[test]
    fn test_score_schedule_rejects_late_dose() {
        let constraints = base_constraints();
        let window = (
            Utc.with_ymd_and_hms(2026, 5, 25, 9, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 5, 25, 17, 0, 0).unwrap(),
        );
        let sleep = Utc.with_ymd_and_hms(2026, 5, 25, 23, 0, 0).unwrap();
        // dose is 1 hour after no_caffeine_after
        let late = vec![Dose {
            time: Utc.with_ymd_and_hms(2026, 5, 25, 21, 0, 0).unwrap(),
            mg: 95.0,
        }];
        assert!(score_schedule(&late, &constraints, window, sleep, 5.0).is_none());
    }

    // ── optimizer end-to-end ────────────────────────────────────────────────

    #[test]
    fn test_optimizer_two_doses() {
        let out = optimize(base_input(vec![95.0, 95.0]));
        assert_eq!(out.recommended_doses.len(), 2);
        assert!(out.sleep_score >= 0.0 && out.sleep_score <= 100.0);
        assert!(!out.alertness_curve.is_empty());
    }

    #[test]
    fn test_optimizer_single_dose() {
        let out = optimize(base_input(vec![100.0]));
        assert_eq!(out.recommended_doses.len(), 1);
        assert!(!out.alertness_curve.is_empty());
    }

    #[test]
    fn test_optimizer_no_doses_graceful() {
        let out = optimize(base_input(vec![]));
        assert!(out.recommended_doses.is_empty());
        // alertness curve should still cover the window (all zeroes)
        assert!(!out.alertness_curve.is_empty());
        assert!((out.sleep_score - 100.0).abs() < 1e-6); // no caffeine = perfect sleep
    }
}
