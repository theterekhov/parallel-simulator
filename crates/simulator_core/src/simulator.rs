use crate::{Resource, Strategy, SystemState, Thread, ThreadStatus, strategy};

pub struct Simulator {
    pub state: SystemState,
}

impl Simulator {
    pub fn new(threads: Vec<Thread>, resources: Vec<Resource>) -> Self {
        Self {
            state: SystemState {
                current_tick: 0,
                is_deadlocked: false,
                threads,
                resources,
                event_log: vec![
                    "[SYSTEM] Среда инициализирована успешно. Алгоритмы диспетчиризации загружены."
                        .to_string(),
                ],
                strategy: Strategy::default(),
                starvation_threshold: 20,
            },
        }
    }

    pub fn from_state(state: SystemState) -> Self {
        Self { state }
    }

    pub fn is_finished(&self) -> bool {
        self.state.is_deadlocked
            || self
                .state
                .threads
                .iter()
                .all(|t| t.status == ThreadStatus::Terminated)
    }

    pub fn tick(&mut self) {
        if self.is_finished() {
            return;
        }

        self.state.current_tick += 1;

        'advance: {
            let Some(idx) = self.find_or_activate_thread() else {
                break 'advance;
            };

            let step_idx = self.state.threads[idx].current_step_index;

            if step_idx >= self.state.threads[idx].steps.len() {
                let thread_id = self.state.threads[idx].id;
                let tick = self.state.current_tick;

                self.state.threads[idx].status = ThreadStatus::Terminated;
                self.state.event_log.push(format!(
                    "[SYSTEM] Такт {tick}: Поток #{thread_id} успешно завершил работу"
                ));

                break 'advance;
            }

            self.state.threads[idx].steps[step_idx].duration = self.state.threads[idx].steps
                [step_idx]
                .duration
                .saturating_sub(1);

            if self.state.threads[idx].steps[step_idx].duration > 0 {
                let thread_id = self.state.threads[idx].id;
                let action = self.state.threads[idx].steps[step_idx].action.clone();
                let dur = self.state.threads[idx].steps[step_idx].duration;
                let tick = self.state.current_tick;

                self.state.event_log.push(format!(
                	"[EXEC] Такт {tick}: Поток #{thread_id} выполняет \"{action}\" (осталось: {dur})"
                ));
            }

            if self.state.threads[idx].steps[step_idx].duration == 0 {
                let action = self.state.threads[idx].steps[step_idx].action.clone();
                let target = self.state.threads[idx].steps[step_idx].target.clone();

                self.execute_step(idx, &action, target);

                if self.state.threads[idx].status == ThreadStatus::Running {
                    self.state.threads[idx].status = ThreadStatus::Ready;
                }
            }
        }

        self.check_deadlock();
        self.check_sheduling_issues();
    }

    fn find_or_activate_thread(&mut self) -> Option<usize> {
        strategy::find_or_activate_thread(
            &mut self.state.threads,
            &self.state.strategy,
            self.state.current_tick,
            &mut self.state.event_log,
        )
    }

    fn execute_step(&mut self, thread_idx: usize, action: &str, target: Option<String>) {
        match action {
            "lock" => self.execute_lock(thread_idx, target),
            "unlock" => self.execute_unlock(thread_idx, target),
            _ => {
                let thread_id = self.state.threads[thread_idx].id;
                let tick = self.state.current_tick;

                self.state.event_log.push(format!(
                    "[EXEC] Такт {tick}: Поток #{thread_id} завершил \"{action}\""
                ));
                self.state.threads[thread_idx].current_step_index += 1;
            }
        }
    }

    fn execute_lock(&mut self, thread_idx: usize, target: Option<String>) {
        let Some(res_id_str) = target else {
            self.state.threads[thread_idx].current_step_index += 1;
            return;
        };

        let res_id = res_id_str.parse().unwrap_or(0);

        let resource_status = self
            .state
            .resources
            .iter()
            .find(|r| r.id == res_id)
            .map(|r| (r.capacity, r.owners.len()));

        match resource_status {
            Some((capacity, owners_count)) => {
                if owners_count < capacity {
                    let thread_id = self.state.threads[thread_idx].id;
                    let tick = self.state.current_tick;

                    if let Some(res) = self.state.resources.iter_mut().find(|r| r.id == res_id) {
                        res.owners.push(thread_id);
                    }

                    self.state.threads[thread_idx].current_step_index += 1;
                    self.state.threads[thread_idx].wait_start_tick = None;

                    self.state.event_log.push(format!(
                        "[SYNC] Такт {tick}: Поток #{thread_id} захватил ресурс #{res_id}"
                    ));
                } else {
                    let thread_id = self.state.threads[thread_idx].id;
                    let tick = self.state.current_tick;
                    let step_idx = self.state.threads[thread_idx].current_step_index;

                    self.state.threads[thread_idx].steps[step_idx].duration = 1;
                    self.state.threads[thread_idx].status = ThreadStatus::Blocked;
                    self.state.threads[thread_idx].wait_start_tick = Some(tick);

                    self.state.event_log.push(format!(
                    	"[BLOCK] Такт {tick}: Поток #{thread_id} заблокирован. Ресурс #{res_id} занят"
                    ));
                }
            }
            None => {
                self.state.threads[thread_idx].current_step_index += 1;
            }
        }
    }

    fn execute_unlock(&mut self, thread_idx: usize, target: Option<String>) {
        let thread_id = self.state.threads[thread_idx].id;
        let tick = self.state.current_tick;

        self.state.threads[thread_idx].current_step_index += 1;

        let Some(res_id_str) = target else {
            return;
        };
        let res_id = res_id_str.parse().unwrap_or(0);

        if let Some(res) = self.state.resources.iter_mut().find(|r| r.id == res_id) {
            res.owners.retain(|&id| id != thread_id);
        }

        self.state.event_log.push(format!(
            "[SYNC] Такт {tick}: Поток #{thread_id} освободил ресурс #{res_id}"
        ));

        for thread in self.state.threads.iter_mut() {
            if thread.status != ThreadStatus::Blocked {
                continue;
            }

            let step_idx = thread.current_step_index;
            if let Some(step) = thread.steps.get(step_idx) {
                if step.action == "lock" {
                    if let Some(t_target) = &step.target {
                        if t_target.parse::<u32>().unwrap_or(u32::MAX) == res_id {
                            thread.status = ThreadStatus::Ready;
                            thread.wait_start_tick = None;

                            let waked_id = thread.id;

                            self.state.event_log.push(format!(
                                "[SHEDULER] Такт {tick}: Поток #{waked_id} разблокирован"
                            ));
                        }
                    }
                }
            }
        }
    }

    pub fn check_deadlock(&mut self) {
        let was_deadlocked = self.state.is_deadlocked;

        let n = self.state.threads.len();
        let mut visited = vec![false; n];
        let mut in_stack = vec![false; n];

        let mut deadlocked = false;

        for start_idx in 0..n {
            if !visited[start_idx] && self.state.threads[start_idx].status == ThreadStatus::Blocked
            {
                if Self::dfs_has_cycle(&self.state, start_idx, &mut visited, &mut in_stack) {
                    deadlocked = true;
                    break;
                }
            }
        }

        self.state.is_deadlocked = deadlocked;

        if deadlocked && !was_deadlocked {
            let tick = self.state.current_tick;

            self.state.event_log.push(format!(
                "[DEADLOCK] Такт {tick}: КРИТИЧЕСКАЯ ОШИБКА: Обнаружена взаимная блокировка"
            ));
        }
    }

    fn dfs_has_cycle(
        state: &SystemState,
        thread_idx: usize,
        visited: &mut Vec<bool>,
        in_stack: &mut Vec<bool>,
    ) -> bool {
        visited[thread_idx] = true;
        in_stack[thread_idx] = true;

        if let Some(next_idx) = Self::get_wait_for_idx(state, thread_idx) {
            if !visited[next_idx] {
                if Self::dfs_has_cycle(state, next_idx, visited, in_stack) {
                    return true;
                }
            } else if in_stack[next_idx] {
                return true;
            }
        }

        in_stack[thread_idx] = false;
        false
    }

    fn get_wait_for_idx(state: &SystemState, thread_idx: usize) -> Option<usize> {
        let thread = &state.threads[thread_idx];

        if thread.status != ThreadStatus::Blocked {
            return None;
        }

        let step = thread.steps.get(thread.current_step_index)?;

        if step.action != "lock" {
            return None;
        }

        let res_id = step.target.as_ref()?.parse::<u32>().ok()?;
        let res = state.resources.iter().find(|r| r.id == res_id)?;
        let owner_id = *res.owners.first()?;

        state.threads.iter().position(|t| t.id == owner_id)
    }

    fn check_sheduling_issues(&mut self) {
        let tick = self.state.current_tick;
        let threshold = self.state.starvation_threshold;

        for thread in &self.state.threads {
            if thread.status != ThreadStatus::Blocked {
                continue;
            }

            if let Some(wait_start) = thread.wait_start_tick {
                let wait_duration = tick.saturating_sub(wait_start);

                if wait_duration > threshold {
                    self.state.event_log.push(format!(
                        "[WARNING] Такт {tick}: Поток #{} голодание ({} тактов)",
                        thread.id, wait_duration
                    ));
                }
            }
        }

        for thread in &self.state.threads {
            if thread.status != ThreadStatus::Blocked {
                continue;
            }

            let high_prio = thread.priority;
            let high_id = thread.id;

            let waiting_res_id = thread
                .steps
                .get(thread.current_step_index)
                .and_then(|s| {
                    if s.action == "lock" {
                        s.target.as_ref()
                    } else {
                        None
                    }
                })
                .and_then(|t| t.parse::<u32>().ok());

            if let Some(res_id) = waiting_res_id {
                if let Some(res) = self.state.resources.iter().find(|r| r.id == res_id) {
                    for owner_id in &res.owners {
                        if let Some(owner) = self.state.threads.iter().find(|t| t.id == *owner_id) {
                            if owner.priority < high_prio {
                                self.state.event_log.push(format!(
                                    "[WARNING] Такт {tick}: Инверсия! \
                 Поток #{} (Prio: {}) ждет #{} (Prio: {})",
                                    high_id, high_prio, owner.id, owner.priority
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Resource, ResourceType, Step, Thread, ThreadStatus};

    fn run_to_completion(sim: &mut Simulator, max_ticks: u64) {
        for _ in 0..max_ticks {
            if sim.is_finished() {
                break;
            }

            sim.tick();
        }
    }

    #[test]
    fn test_basic_execution() {
        let threads = vec![Thread {
            id: 1,
            priority: 1,
            status: ThreadStatus::Ready,
            current_step_index: 0,
            wait_start_tick: None,
            last_ready_tick: 0,
            steps: vec![Step {
                action: "compute".to_string(),
                target: None,
                duration: 2,
            }],
        }];

        let mut sim = Simulator::new(threads, vec![]);

        run_to_completion(&mut sim, 10);

        assert!(sim.is_finished());
        assert!(!sim.state.is_deadlocked);
        assert_eq!(sim.state.threads[0].status, ThreadStatus::Terminated);
    }

    #[test]
    fn test_deadlock_detection() {
        let resources = vec![
            Resource {
                id: 1,
                res_type: ResourceType::Mutex,
                capacity: 1,
                owners: vec![],
            },
            Resource {
                id: 2,
                res_type: ResourceType::Mutex,
                capacity: 1,
                owners: vec![],
            },
        ];

        let threads = vec![
            Thread {
                id: 1,
                priority: 1,
                status: ThreadStatus::Ready,
                current_step_index: 0,
                wait_start_tick: None,
                last_ready_tick: 0,
                steps: vec![
                    Step {
                        action: "lock".to_string(),
                        target: Some("1".to_string()),
                        duration: 1,
                    },
                    Step {
                        action: "lock".to_string(),
                        target: Some("2".to_string()),
                        duration: 1,
                    },
                ],
            },
            Thread {
                id: 2,
                priority: 1,
                status: ThreadStatus::Ready,
                current_step_index: 0,
                wait_start_tick: None,
                last_ready_tick: 0,
                steps: vec![
                    Step {
                        action: "lock".to_string(),
                        target: Some("2".to_string()),
                        duration: 1,
                    },
                    Step {
                        action: "lock".to_string(),
                        target: Some("1".to_string()),
                        duration: 1,
                    },
                ],
            },
        ];

        let mut sim = Simulator::new(threads, resources);
        sim.state.strategy = Strategy::CPthreads;

        run_to_completion(&mut sim, 20);

        assert!(sim.is_finished());
        assert!(
            sim.state.is_deadlocked,
            "Симулятор должен был обнаружить взаимную блокирвоку"
        );
        assert_eq!(sim.state.threads[0].status, ThreadStatus::Blocked);
        assert_eq!(sim.state.threads[1].status, ThreadStatus::Blocked);
    }

    #[test]
    fn test_semaphore_capacity() {
        let resources = vec![Resource {
            id: 1,
            res_type: ResourceType::Semaphore,
            capacity: 2,
            owners: vec![],
        }];

        let create_thread = |id| Thread {
            id,
            priority: 1,
            status: ThreadStatus::Ready,
            current_step_index: 0,
            wait_start_tick: None,
            last_ready_tick: 0,
            steps: vec![
                Step {
                    action: "lock".to_string(),
                    target: Some("1".to_string()),
                    duration: 1,
                },
                Step {
                    action: "compute".to_string(),
                    target: None,
                    duration: 10,
                },
            ],
        };

        let mut sim = Simulator::new(
            vec![create_thread(1), create_thread(2), create_thread(3)],
            resources,
        );

        for _ in 0..10 {
            sim.tick();
        }

        assert_eq!(
            sim.state.resources[0].owners.len(),
            2,
            "Семафор должен впустить  ровно 2 потока"
        );

        let blocked_count = sim
            .state
            .threads
            .iter()
            .filter(|t| t.status == ThreadStatus::Blocked)
            .count();

        assert_eq!(blocked_count, 1, "Третий поток должен быть заблокирован");
    }
}
