use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub max_daily_mg: f64,
    pub min_gap_hours: f64,
    pub no_caffeine_after: DateTime<Utc>,
}
