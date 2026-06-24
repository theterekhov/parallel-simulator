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
        total_ticks: state.current_tick as u64,
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

pub fn download_json(json_str: &str, filename: &str) {
    let parts = web_sys::js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(json_str));

    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("application/json;charset=utf-8");

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts)
        .expect("Не удалось создать Blob");

    let url =
        web_sys::Url::create_object_url_with_blob(&blob).expect("Не удалось создать object URL");

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
    anchor.set_download(filename);
    anchor.style().set_css_text("display: none;");

    document.body().unwrap().append_child(&anchor).unwrap();
    anchor.click();
    document.body().unwrap().remove_child(&anchor).unwrap();

    web_sys::Url::revoke_object_url(&url).unwrap();
}

pub fn download_csv(log: &[String]) {
    let mut csv_content = String::from("tick,type,message\n");

    for entry in log {
        let prefix = "Такт ";
        let (tick, msg) = if let (Some(a), Some(b)) = (entry.find(prefix), entry.find(": ")) {
            let after_prefix = &entry[a + prefix.len()..b];
            let tick_str = after_prefix.trim().to_string();
            (tick_str, entry[b + 2..].to_string())
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

    let parts = web_sys::js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(&csv_content));

    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("text/csv;charset=utf-8");

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts)
        .expect("не удалось создать Blob");

    let url =
        web_sys::Url::create_object_url_with_blob(&blob).expect("не удалось создать Object URL");

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
    anchor.set_download("simulation_report.csv");
    anchor.style().set_css_text("display: none;");

    document.body().unwrap().append_child(&anchor).unwrap();
    anchor.click();
    document.body().unwrap().remove_child(&anchor).unwrap();

    web_sys::Url::revoke_object_url(&url).unwrap();
}
