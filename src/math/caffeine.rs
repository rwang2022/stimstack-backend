use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Dose {
    pub mg: f64, 
    pub time: DateTime<Utc>,
}

// Computes caffeine decay using half-life model
pub fn caffeine_decay(c0: f64, hours: f64) -> f64 {
    let k = (2.0_f64).ln() / 5.0;
    c0 * (-k * hours).exp()
}

// Computes total caffeine in bloodstream at time t
pub fn total_caffeine(doses: &[Dose], t: DateTime<Utc>) -> f64 {
    doses
        .iter()
        .map(|dose| {
            let dt = (t - dose.time).num_minutes() as f64 / 60.0;
            if dt < 0.0 {
                0.0
            } else {
                caffeine_decay(dose.mg, dt)
            }
        })
        .sum()
}