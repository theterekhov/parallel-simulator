use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    Mutex,
    Semaphore,
}

impl ResourceType {
    pub fn description(&self) -> &'static str {
        match self {
            ResourceType::Mutex => "Мьютекс (владеет 1 поток)",
            ResourceType::Semaphore => "Семафор (владеют N потоков)",
        }
    }
}

fn default_capacity() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: u32,
    pub res_type: ResourceType,
    #[serde(default = "default_capacity")]
    pub capacity: usize,
    #[serde(default)]
    pub owners: Vec<u32>,
}

impl Resource {
    pub fn can_acquire(&self) -> bool {
        self.owners.len() < self.capacity
    }

    pub fn acquire(&mut self, thread_id: u32) -> bool {
        if !self.can_acquire() {
            return false;
        }

        self.owners.push(thread_id);
        true
    }

    pub fn release(&mut self, thread_id: u32) -> bool {
        if let Some(pos) = self.owners.iter().position(|&id| id == thread_id) {
            self.owners.remove(pos);
            true
        } else {
            false
        }
    }
}
