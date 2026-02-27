use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityLevel {
    Sedentary,
    Moderate,
    Athletic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub age: u8,             // years
    pub weight_kg: f64,      // kg
    pub height_cm: f64,      // cm
    pub sex: Sex,
    pub activity_level: ActivityLevel,
    pub smoker: bool,
}
