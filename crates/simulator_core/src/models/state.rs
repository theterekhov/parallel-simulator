use serde::{Deserialize, Serialize};

use super::{Resource, Strategy, Thread};

fn default_starvation_threshold() -> u64 {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub current_tick: u64,
    pub is_deadlocked: bool,
    pub threads: Vec<Thread>,
    #[serde(default)]
    pub resources: Vec<Resource>,
    #[serde(default)]
    pub event_log: Vec<String>,
    #[serde(default)]
    pub strategy: Strategy,
    #[serde(default = "default_starvation_threshold")]
    pub starvation_threshold: u64,
}
