use simulator_core::{Simulator, ThreadStatus};
use wasm_bindgen::JsCast;

#[derive(Debug, Clone)]
pub struct SimulationReport {
    pub status: String,
    pub total_ticks: u64,
    pub threads_completed: usize,
    pub threads_total: usize,
    pub deadlock_detected: bool,
    pub resources_used: Vec<(u32, usize, usize)>,
    pub event_count: usize,
}

pub fn generate_report(sim: &Simulator) -> SimulationReport {
    let state = &sim.state;

    let threads_completed = state
        .threads
        .iter()
        .filter(|t| t.status == ThreadStatus::Terminated)
        .count();

    let threads_total = state.threads.len();

    let status = if state.is_deadlocked {
        "Взаимная блокировка (Deadlock)".to_string()
    } else if threads_completed == state.threads.len() {
        "Успешно завершено".to_string()
    } else {
        "Прервано".to_string()
    };

    let resources_used = state
        .resources
        .iter()
        .map(|r| (r.id, r.owners.len(), r.capacity))
        .collect();

    SimulationReport {
        status,
        total_ticks: state.current_tick,
        threads_completed,
        threads_total,
        deadlock_detected: state.is_deadlocked,
        resources_used,
        event_count: state.event_log.len(),
    }
}

pub fn download_report(log: &[String]) {
    let content = log.join("\n");

    let parts = web_sys::js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(&content));

    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("text/plain;charset=utf-8");

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts)
        .expect("Не удалось создать Blob");

    let url =
        web_sys::Url::create_object_url_with_blob(&blob).expect("Не удалось создать Object URL");

    let document = web_sys::window()
        .expect("Нет window")
        .document()
        .expect("Нет document");

    let anchor = document
        .create_element("a")
        .expect("Не удалось создать элемент \"a\"")
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .expect("Не HtmlAnchorElement");

    anchor.set_href(&url);
    anchor.set_download("simulation_report.txt");
    anchor.style().set_css_text("display: none;");

    document.body().unwrap().append_child(&anchor).unwrap();
    anchor.click();
    document.body().unwrap().remove_child(&anchor).unwrap();

    web_sys::Url::revoke_object_url(&url).unwrap();
}

fn download_blob(content: &str, mime: &str, filename: &str) {
    let parts = web_sys::js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(content));

    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime);

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts);
    let blob = match blob {
        Ok(b) => b,
        Err(_) => return,
    };

    let url = web_sys::Url::create_object_url_with_blob(&blob);
    let url = match url {
        Ok(u) => u,
        Err(_) => return,
    };

    let document = match web_sys::window().and_then(|w| w.document()) {
        Some(d) => d,
        None => return,
    };

    let anchor = match document.create_element("a") {
        Ok(el) => match el.dyn_into::<web_sys::HtmlAnchorElement>() {
            Ok(a) => a,
            Err(_) => return,
        },
        Err(_) => return,
    };

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.style().set_css_text("display: none;");

    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    }

    let _ = web_sys::Url::revoke_object_url(&url);
}

pub fn download_json(json_str: &str, filename: &str) {
    download_blob(json_str, "application/json;charset=utf-8", filename);
}

pub fn download_csv(log: &[String]) {
    let mut csv_content = String::from("tick,type,message\n");

    for entry in log {
        let (tick, msg) = if let (Some(a), Some(b)) = (entry.find("Такт "), entry.find(": ")) {
            let after_prefix = &entry[a + "Такт ".len()..b];
            (after_prefix.trim().to_string(), entry[b + 2..].to_string())
        } else if let (Some(a), Some(b)) = (entry.find("Tick "), entry.find(": ")) {
            let after_prefix = &entry[a + "Tick ".len()..b];
            (after_prefix.trim().to_string(), entry[b + 2..].to_string())
        } else {
            ("0".to_string(), entry.clone())
        };

        let event_type = if entry.starts_with("[SYSTEM]") {
            "SYSTEM"
        } else if entry.starts_with("[EXEC]") {
            "EXEC"
        } else if entry.starts_with("[SYNC]") {
            "SYNC"
        } else if entry.starts_with("[SCHEDULER]") {
            "SCHEDULER"
        } else if entry.starts_with("[DEADLOCK]") {
            "DEADLOCK"
        } else if entry.starts_with("[WARNING]") {
            "WARNING"
        } else {
            "INFO"
        };

        csv_content.push_str(&format!(
            "{},{},\"{}\"\n",
            tick,
            event_type,
            msg.replace('"', "'")
        ));
    }

    download_blob(
        &csv_content,
        "text/csv;charset=utf-8",
        "simulation_report.csv",
    );
}
