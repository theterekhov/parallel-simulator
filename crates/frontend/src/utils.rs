use simulator_core::{Simulator, ThreadStatus};

pub fn generate_svg(sim: &Simulator) -> String {
    let threads = &sim.state.threads;
    let resources = &sim.state.resources;
    let deadlocked = sim.state.is_deadlocked;

    let n_t = threads.len().max(1);
    let n_r = resources.len().max(1);
    let n_max = n_t.max(n_r);

    let h: i32 = ((n_max as i32) * 80 + 80).max(300);
    let w: i32 = 520;

    let t_step = (h - 80) / n_t as i32;
    let thread_pos: Vec<(u32, i32, ThreadStatus)> = threads
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id, 40 + t_step / 2 + i as i32 * t_step, t.status.clone()))
        .collect();

    let r_step = (h - 80) / n_r as i32;
    let res_pos: Vec<(u32, i32)> = resources
        .iter()
        .enumerate()
        .map(|(i, r)| (r.id, 40 + r_step / 2 + i as i32 * r_step))
        .collect();

    let border_color = if deadlocked { "#dc2626" } else { "#e2e8f0" };
    let border_width = if deadlocked { 3 } else { 1 };

    let mut s = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='100%' height='{h}' \
             viewBox='0 0 {w} {h}' \
             style='background:#ffffff;border:{border_width}px solid {border_color};\
                    border-radius:8px;display:block;'>\n"
    );

    s += "<defs>\
              <marker id='arr-g' markerWidth='10' markerHeight='7' refX='9' refY='3.5' orient='auto'>\
                <polygon points='0 0, 10 3.5, 0 7' fill='#22c55e'/>\
              </marker>\
              <marker id='arr-r' markerWidth='10' markerHeight='7' refX='9' refY='3.5' orient='auto'>\
                <polygon points='0 0, 10 3.5, 0 7' fill='#ef4444'/>\
              </marker>\
            </defs>\n";

    s += "<text x='70' y='20' text-anchor='middle' fill='#64748b' \
              font-size='11' font-weight='700'>ПОТОКИ</text>\n";
    s += "<text x='430' y='20' text-anchor='middle' fill='#64748b' \
              font-size='11' font-weight='700'>РЕСУРСЫ</text>\n";

    for res in resources.iter() {
        if let Some(&owner_id) = res.owners.first() {
            let r_cy = res_pos
                .iter()
                .find(|&&(rid, _)| rid == res.id)
                .map(|&(_, cy)| cy);
            let t_cy = thread_pos
                .iter()
                .find(|&&(tid, _, _)| tid == owner_id)
                .map(|&(_, cy, _)| cy);

            if let (Some(r_cy), Some(t_cy)) = (r_cy, t_cy) {
                let mx = (390 + 99) / 2;
                s += &format!(
                    "<path d='M390,{r_cy} C{mx},{r_cy} {mx},{t_cy} 99,{t_cy}' \
                         fill='none' stroke='#22c55e' stroke-width='2' marker-end='url(#arr-g)'/>\n"
                );
            }
        }
    }

    for t in threads.iter() {
        if t.status != ThreadStatus::Blocked {
            continue;
        }
        if let Some(step) = t.steps.get(t.current_step_index) {
            if step.action == "lock" {
                if let Some(Ok(res_id)) = step.target.as_ref().map(|s| s.parse::<u32>()) {
                    let t_cy = thread_pos
                        .iter()
                        .find(|&&(tid, _, _)| tid == t.id)
                        .map(|&(_, cy, _)| cy);
                    let r_cy = res_pos
                        .iter()
                        .find(|&&(rid, _)| rid == res_id)
                        .map(|&(_, cy)| cy);

                    if let (Some(t_cy), Some(r_cy)) = (t_cy, r_cy) {
                        let mx = (99 + 390) / 2;
                        s += &format!(
                            "<path d='M99,{t_cy} C{mx},{t_cy} {mx},{r_cy} 390,{r_cy}' \
                                 fill='none' stroke='#ef4444' stroke-width='1.5' \
                                 stroke-dasharray='6,3' marker-end='url(#arr-r)'/>\n"
                        );
                    }
                }
            }
        }
    }

    for &(tid, cy, ref status) in thread_pos.iter() {
        let (fill, ring) = match status {
            ThreadStatus::Running => ("#2563eb", "#1d4ed8"),
            ThreadStatus::Blocked => ("#dc2626", "#b91c1c"),
            ThreadStatus::Ready => ("#16a34a", "#15803d"),
            ThreadStatus::Terminated => ("#94a3b8", "#64748b"),
            ThreadStatus::New => ("#7c3aed", "#6d28d9"),
        };
        let status_str = status.as_ru_str();

        s += &format!(
            "<circle cx='70' cy='{cy}' r='28' fill='{fill}' stroke='{ring}' stroke-width='2.5'/>\n\
                 <text x='70' y='{tly}' text-anchor='middle' fill='white' \
                 font-size='14' font-weight='700'>T{tid}</text>\n\
                 <text x='70' y='{sly}' text-anchor='middle' fill='#475569' \
                 font-size='9'>{status_str}</text>\n",
            tly = cy + 5,
            sly = cy + 43,
        );
    }

    let rw = 80i32;
    let rh = 36i32;

    for &(rid, cy) in res_pos.iter() {
        let res = resources.iter().find(|r| r.id == rid).unwrap();
        let is_full = res.owners.len() >= res.capacity;
        let (fill, ring) = if is_full {
            ("#fef3c7", "#d97706")
        } else {
            ("#f1f5f9", "#94a3b8")
        };
        let type_str = if res.capacity > 1 {
            "Семафор"
        } else {
            "Мьютекс"
        };
        let usage_str = format!("{}/{}", res.owners.len(), res.capacity);

        let rx = 430 - rw / 2;
        let ry = cy - rh / 2;

        s += &format!(
            "<rect x='{rx}' y='{ry}' width='{rw}' height='{rh}' rx='5' \
                 fill='{fill}' stroke='{ring}' stroke-width='2'/>\n\
                 <text x='430' y='{tly}' text-anchor='middle' fill='#334155' \
                 font-size='13' font-weight='700'>R{rid}</text>\n\
                 <text x='430' y='{uly}' text-anchor='middle' fill='#64748b' \
                 font-size='9'>{type_str} · {usage_str}</text>\n",
            tly = cy + 5,
            uly = cy + rh / 2 + 16,
        );
    }

    if deadlocked {
        let cx = w / 2;
        s += &format!(
            "<rect x='0' y='0' width='{w}' height='{h}' fill='none' stroke='#dc2626' \
                 stroke-width='4' rx='7' opacity='0.7'/>\n\
                 <text x='{cx}' y='{hy}' text-anchor='middle' fill='#dc2626' \
                 font-size='12' font-weight='700' letter-spacing='1'>ВЗАИМНАЯ БЛОКИРОВКА</text>\n",
            hy = h - 12,
        );
    }

    s += "</svg>";
    s
}
