use crate::{Strategy, Thread, ThreadStatus};

pub fn find_or_activate_thread(
    threads: &mut [Thread],
    strategy: &Strategy,
    current_tick: u64,
    event_log: &mut Vec<String>,
) -> Option<usize> {
    match strategy {
        Strategy::CPthreads => pthreads_pick(threads, current_tick, event_log),
        Strategy::PythonGil => gil_pick(threads, current_tick, event_log),
        Strategy::GoChannels => channels_pick(threads, current_tick, event_log),
    }
}

fn pthreads_pick(threads: &mut [Thread], tick: u64, log: &mut Vec<String>) -> Option<usize> {
    let candidates = threads
        .iter()
        .enumerate()
        .filter(|(_, t)| t.status == ThreadStatus::Ready || t.status == ThreadStatus::Running)
        .map(|(i, _)| i)
        .collect::<Vec<usize>>();

    if candidates.is_empty() {
        return None;
    }

    let idx = candidates[tick as usize % candidates.len()];

    for &i in &candidates {
        if i == idx {
            if threads[i].status != ThreadStatus::Running {
                log.push(format!(
                    "[SCHEDULER] Такт {tick}: Поток #{} получил квант процессорного времени",
                    threads[i].id,
                ));
            }

            threads[i].status = ThreadStatus::Running;
        } else if threads[i].status == ThreadStatus::Running {
            log.push(format!(
        	"[SCHEDULER] Такт {tick}: Поток #{} принудительно вытеснен планировщиком (Round-Robin)", threads[i].id
        ));

            threads[i].status = ThreadStatus::Ready;
        }
    }

    Some(idx)
}

fn gil_pick(threads: &mut [Thread], tick: u64, log: &mut Vec<String>) -> Option<usize> {
    if let Some(idx) = threads
        .iter()
        .position(|t| t.status == ThreadStatus::Running)
    {
        return Some(idx);
    }

    let idx = threads
        .iter()
        .position(|t| t.status == ThreadStatus::Ready)?;

    threads[idx].status = ThreadStatus::Running;

    log.push(format!(
        "[SCHEDULER] Такт {tick}: Поток #{} захватил GIL и получил квант времени",
        threads[idx].id
    ));

    Some(idx)
}

fn channels_pick(threads: &mut [Thread], tick: u64, log: &mut Vec<String>) -> Option<usize> {
    let candidates = threads
        .iter()
        .enumerate()
        .filter(|(_, t)| t.status == ThreadStatus::Ready || t.status == ThreadStatus::Running)
        .map(|(i, _)| i)
        .collect::<Vec<usize>>();

    if candidates.is_empty() {
        return None;
    }

    let idx = *candidates
        .iter()
        .min_by_key(|&&i| threads[i].last_ready_tick)
        .unwrap();

    for &i in &candidates {
        if i == idx {
            if threads[i].status != ThreadStatus::Running {
                log.push(format!(
	            	"[SCHEDULER] Такт {tick}: Поток #{} получил квант (Go Channels - честное планирование)",threads[i].id
	            ));
            }

            threads[i].status = ThreadStatus::Running;
        } else if threads[i].status == ThreadStatus::Running {
            log.push(format!(
                "[SCHEDULER] Такт {tick}: Поток #{} вытеснен (Go Channels)",
                threads[i].id
            ));

            threads[i].status = ThreadStatus::Ready;
            threads[i].last_ready_tick = tick;
        }
    }

    Some(idx)
}
