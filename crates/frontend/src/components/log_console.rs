use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::models::simulation_report::{download_csv, download_report};

#[component]
pub fn LogConsole() -> impl IntoView {
    let simulator = use_context::<RwSignal<Option<simulator_core::Simulator>>>()
        .expect("Контекст симулятора не найден");
    let export_format = RwSignal::new(String::from("txt"));

    let on_download = move |_| {
        simulator.with(|opt| {
            if let Some(sim) = opt {
                let format = export_format.get();

                if format == "csv" {
                    download_csv(&sim.state.event_log);
                } else {
                    download_report(&sim.state.event_log);
                }
            }
        });
    };

    view! {
        <div class="log-console">
            <div class="log-header">
                <div>
                    <span style="font-weight: bold; color: var(--muted);">"Журнал событий"</span>
                    <span style="font-size: 0.75rem; color: var(--muted); margin-left: 8px;">
                        {move || simulator.with(|opt| opt.as_ref().map(|s| s.state.event_log.len()).unwrap_or(0))}
                        " записей"
                    </span>
                </div>
                <div style="display: flex; gap: 8px; align-items: center;">
                    <select
                        class="export-format-select"
                        on:change=move |ev| {
                            if let Some(target) = ev.target()
                                && let Ok(select) = target.dyn_into::<web_sys::HtmlSelectElement>() {
                                    export_format.set(select.value());
                                }
                        }
                        prop:value=move || export_format.get()
                    >
                        <option value="txt">"TXT"</option>
                        <option value="csv">"CSV"</option>
                    </select>
                    <button
                        class="btn-report"
                        on:click=on_download
                        disabled=move || simulator.with(|opt| opt.as_ref().map(|s| s.state.event_log.is_empty()).unwrap_or(true))
                    >
                        "Выгрузить"
                    </button>
                </div>
            </div>
            <div class="log-entries">
                {move || {
                    simulator
                        .with(|opt| {
                            opt.as_ref()
                                .map(|sim| {
                                    let events: Vec<_> = sim.state.event_log.iter().rev().take(50).collect();
                                    events
                                        .into_iter()
                                        .map(|entry| {
                                            let text = entry.clone();
                                            let color_class = if entry.contains("DEADLOCK") || entry.contains("БЛОКИРОВКА") {
                                                "log-entry log-entry-error"
                                            } else if entry.contains("WARNING") || entry.contains("голодание") {
                                                "log-entry log-entry-warning"
                                            } else if entry.contains("Terminated") || entry.contains("завершил") {
                                                "log-entry log-entry-success"
                                            } else if entry.contains("Running") {
                                                "log-entry log-entry-running"
                                            } else if entry.contains("Blocked") || entry.contains("блокирован") {
                                                "log-entry log-entry-blocked"
                                            } else {
                                                "log-entry"
                                            };
                                            view! {
                                                <div class={color_class}>
                                                    {text}
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        })
                }}
            </div>
        </div>
    }
}
