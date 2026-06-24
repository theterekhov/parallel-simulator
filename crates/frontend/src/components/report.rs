use leptos::prelude::*;

use crate::models::simulation_report::{SimulationReport, download_report};

#[component]
pub fn ReportModal() -> impl IntoView {
    let show_report = use_context::<RwSignal<Option<SimulationReport>>>()
        .expect("Контекст show_report не найден");
    let simulator = use_context::<RwSignal<Option<simulator_core::Simulator>>>()
        .expect("Контекст симулятора не найден");

    view! {
        {move || {
            show_report
                .get()
                .map(|report| {
                    let status_class = if report.deadlock_detected {
                        "report-status-deadlock"
                    } else {
                        "report-status-success"
                    };
                    let sim_clone = simulator;
                    view! {
                        <div class="modal-backdrop" on:click=move |_| { show_report.set(None) }>
                            <div
                                class="modal-content"
                                style="max-width: 500px; max-height: 80vh; overflow-y: auto;"
                                on:click=move |e| e.stop_propagation()
                            >
                                <h2 style="margin-bottom: 20px;">"Отчет о симуляции"</h2>
                                <p>
                                    <strong>"Статус: "</strong>
                                    <span class=status_class>{report.status.clone()}</span>
                                </p>
                                <p>
                                    <strong>"Всего тактов: "</strong>
                                    {report.total_ticks}
                                </p>
                                <p>
                                    <strong>"Потоки: "</strong>
                                    {format!(
                                        "{} из {} завершены",
                                        report.threads_completed,
                                        report.threads_total,
                                    )}
                                </p>
                                <p>
                                    <strong>"Событий в журнале: "</strong>
                                    {report.event_count}
                                </p>
                                <h3 style="margin-top: 20px; margin-bottom: 10px;">"Использование ресурсов"</h3>
                                <ul style="list-style: none; padding: 0; margin-bottom: 20px;">
                                    {report
                                        .resources_used
                                        .iter()
                                        .map(|(id, owners, capacity)| {
                                            let text = format!("Ресурс {id}: {owners}/{capacity} занято");
                                            view! {
                                                <li style="padding: 5px 0; border-bottom: 1px solid var(--border);">
                                                    {text}
                                                </li>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </ul>
                                <div class="modal-buttons">
                                    <button
                                        class="btn-secondary"
                                        on:click=move |_| {
                                            sim_clone
                                                .with(|opt| {
                                                    if let Some(sim) = opt {
                                                        let log = sim.state.event_log.clone();
                                                        download_report(&log);
                                                    }
                                                });
                                            show_report.set(None);
                                        }
                                    >
                                        "Скачать журнал"
                                    </button>
                                    <button
                                        class="btn-primary"
                                        on:click=move |_| { show_report.set(None) }
                                    >
                                        "Закрыть"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                })
        }}
    }
}
