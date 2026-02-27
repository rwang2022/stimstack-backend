use serde::{Deserialize, Serialize};
use crate::model::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSensitivity {
    pub half_life_hours: f64,
    pub sleep_decay_mg: f64,
    pub crash_threshold: f64,
}

impl Default for UserSensitivity {
    fn default() -> Self {
        Self {
            half_life_hours: 5.0,
            sleep_decay_mg: 50.0,
            crash_threshold: 10.0,
        }
    }
} // use with let sensitivity = UserSensitivity::default();

impl UserSensitivity {
    pub fn from_profile(p: &UserProfile) -> Self {
        let mut half_life = 5.0;
        let mut sleep_decay = 50.0;
        let crash_threshold = 10.0;

        // Weight
        half_life *= (p.weight_kg / 70.0).powf(0.25);

        // Age
        if p.age > 25 {
            half_life *= 1.0 + (p.age as f64 - 25.0) * 0.005;
            sleep_decay *= 1.0 - (p.age as f64 - 30.0).max(0.0) * 0.003;
        }

        // Sex
        if p.sex == Sex::Female {
            half_life *= 1.1;
        }

        // Activity
        half_life *= match p.activity_level {
            ActivityLevel::Sedentary => 1.05,
            ActivityLevel::Moderate => 1.0,
            ActivityLevel::Athletic => 0.9,
        };

        sleep_decay *= match p.activity_level {
            ActivityLevel::Sedentary => 0.9,
            ActivityLevel::Moderate => 1.0,
            ActivityLevel::Athletic => 1.1,
        };

        // Smoking
        if p.smoker {
            half_life *= 0.7;
        }

        Self {
            half_life_hours: half_life.clamp(3.0, 8.0),
            sleep_decay_mg: sleep_decay.clamp(30.0, 90.0),
            crash_threshold,
        }
    }
}