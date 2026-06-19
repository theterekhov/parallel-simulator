use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub action: String,
    pub target: Option<String>,
    pub duration: u32,
}
