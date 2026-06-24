use serde::{Deserialize, Serialize};

use super::Step;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreadStatus {
    New,
    Ready,
    Running,
    Blocked,
    Terminated,
}

impl ThreadStatus {
    pub fn as_ru_str(&self) -> &'static str {
        match self {
            ThreadStatus::New => "Новый",
            ThreadStatus::Ready => "Готов",
            ThreadStatus::Running => "Выполняется",
            ThreadStatus::Blocked => "Заблокирован",
            ThreadStatus::Terminated => "Завершен",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: u32,
    pub priority: u32,
    pub status: ThreadStatus,
    pub current_step_index: usize,
    #[serde(default)]
    pub wait_start_tick: Option<u64>,
    #[serde(default)]
    pub last_ready_tick: u64,
    pub steps: Vec<Step>,
}
