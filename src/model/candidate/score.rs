use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScoreComponent {
    pub name: String,
    pub value: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScoreBreakdown {
    pub components: Vec<ScoreComponent>,
    pub final_score: i64,
}
