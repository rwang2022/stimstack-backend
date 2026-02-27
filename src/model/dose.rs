use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dose {
    pub mg: f64,
    pub time: DateTime<Utc>,
}