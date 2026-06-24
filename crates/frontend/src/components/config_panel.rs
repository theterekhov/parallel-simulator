use gloo_net::http::Request;
use leptos::prelude::*;
use simulator_core::{Simulator, Strategy, SystemState};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{Event, HtmlInputElement, HtmlSelectElement};

use crate::models::{
    simulation_report::download_json,
    toast::{Toast, ToastType, push_toast},
};

fn create_step(action: &str, target: Option<&str>, duration: u32) -> simulator_core::Step {
    simulator_core::Step {
        action: action.to_string(),
        target: target.map(|s| s.to_string()),
        duration,
    }
}

fn create_thread(id: u32, steps: Vec<simulator_core::Step>) -> simulator_core::Thread {
    simulator_core::Thread {
        id,
        priority: 1,
        status: simulator_core::ThreadStatus::Ready,
        current_step_index: 0,
        wait_start_tick: None,
        last_ready_tick: 0,
        steps,
    }
}

fn create_mutex(id: u32) -> simulator_core::Resource {
    simulator_core::Resource {
        id,
        res_type: simulator_core::ResourceType::Mutex,
        capacity: 1,
        owners: vec![],
    }
}

fn create_semaphore(id: u32, capacity: usize) -> simulator_core::Resource {
    simulator_core::Resource {
        id,
        res_type: simulator_core::ResourceType::Semaphore,
        capacity,
        owners: vec![],
    }
}

#[component]
pub fn ConfigPanel(tasks: LocalResource<Option<Vec<String>>>) -> impl IntoView {
    let simulator =
        use_context::<RwSignal<Option<Simulator>>>().expect("Контекст симулятора не найден");
    let is_auto_playing = use_context::<RwSignal<bool>>().expect("Контекст авто-режима не найден");
    let task_title = use_context::<RwSignal<String>>().expect("Контекст task_title не найден");
    let task_desc = use_context::<RwSignal<String>>().expect("Контекст task_desc не найден");
    let error_msg = use_context::<RwSignal<String>>().expect("Контекст error_msg не найден");
    let toasts = use_context::<RwSignal<Vec<Toast>>>().expect("Контекст toasts не найден");

    let gen_template = RwSignal::new(String::from("philosophers"));
    let gen_count = RwSignal::new(5_u32);

    let load_task = move |id: String| {
        is_auto_playing.set(false);
        let task_title = task_title;
        let task_desc = task_desc;
        let error_msg = error_msg;
        let toasts = toasts;
        let id = id.clone();

        spawn_local(async move {
            let url = format!("/api/tasks/{}", id);

            let Ok(resp) = Request::get(&url).send().await else {
                error_msg.set("Ошибка загрузки файла".to_string());

                push_toast(toasts, "Ошибка загрузки задачи", ToastType::Error);

                return;
            };

            let Ok(json) = resp.json::<serde_json::Value>().await else {
                error_msg.set("Ошибка парсинга JSON".to_string());

                push_toast(toasts, "Ошибка парсинга JSON", ToastType::Error);

                return;
            };

            let Some(state_val) = json.get("initial_state") else {
                error_msg.set("Отсутствует поле \"initial_state\" в JSON".to_string());

                push_toast(toasts, "Неверный формат задачи", ToastType::Error);

                return;
            };

            let Ok(state) = serde_json::from_value::<SystemState>(state_val.clone()) else {
                error_msg.set("Ошибка десериализации состояния".to_string());

                push_toast(toasts, "Ошибка десериализации", ToastType::Error);

                return;
            };

            if let Some(meta) = json.get("metadata") {
                task_title.set(
                    meta.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                );

                task_desc.set(
                    meta.get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                );
            };
            error_msg.set(String::new());

            simulator.set(Some(Simulator::from_state(state)));

            push_toast(
                toasts,
                format!("Задача \"{}\" загружена", id),
                ToastType::Success,
            );
        });
    };

    let on_file_change = move |event: Event| {
        let error_msg = error_msg;
        let Some(target) = event.target() else {
            return;
        };
        let Ok(input) = target.dyn_into::<HtmlInputElement>() else {
            return;
        };
        let files = input.files();
        let toasts = toasts.clone();

        if let Some(file_list) = files {
            if let Some(file) = file_list.get(0) {
                let is_auto_playing = is_auto_playing.clone();
                let simulator = simulator.clone();
                let input = input.clone();
                let task_title = task_title;
                let task_desc = task_desc;
                let error_msg = error_msg;
                let toasts = toasts.clone();
                let file_name = file.name();

                spawn_local(async move {
                    let text_promise = file.text();
                    let text_result = JsFuture::from(text_promise).await;

                    match text_result {
                        Ok(text_js) => {
                            if let Some(text) = text_js.as_string() {
                                match serde_json::from_str::<serde_json::Value>(&text) {
                                    Ok(json) => {
                                        let validate_resp = Request::post("/api/tasks/validate")
                                            .json(&json)
                                            .unwrap()
                                            .send()
                                            .await;

                                        if let Ok(resp) = validate_resp {
                                            if !resp.ok() {
                                                error_msg.set(
                                                    "Сервер отклонил файл: неверный формат"
                                                        .to_string(),
                                                );

                                                push_toast(
                                                    toasts,
                                                    "Неверный формат файла",
                                                    ToastType::Error,
                                                );

                                                return;
                                            }
                                        }

                                        if let Some(state_val) = json.get("initial_state") {
                                            if let Ok(state) = serde_json::from_value::<SystemState>(
                                                state_val.clone(),
                                            ) {
                                                if let Some(meta) = json.get("metadata") {
                                                    task_title.set(
                                                        meta.get("title")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("")
                                                            .to_string(),
                                                    );

                                                    task_desc.set(
                                                        meta.get("description")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("")
                                                            .to_string(),
                                                    );

                                                    error_msg.set(String::new());
                                                    is_auto_playing.set(false);

                                                    simulator
                                                        .set(Some(Simulator::from_state(state)));

                                                    push_toast(
                                                        toasts,
                                                        format!("Файл \"{}\" загружен", file_name),
                                                        ToastType::Success,
                                                    )
                                                }
                                            } else {
                                                error_msg.set("Ошибка загрузки файла".to_string());

                                                push_toast(
                                                    toasts,
                                                    "Ошибка загрузки файла",
                                                    ToastType::Error,
                                                );
                                            }
                                        } else {
                                            error_msg
                                                .set("Отсутствует \"initial_state\"".to_string());

                                            push_toast(
                                                toasts,
                                                "Ошибка: отсутствует \"initial_state\"",
                                                ToastType::Error,
                                            );
                                        }
                                    }
                                    Err(_) => {
                                        error_msg.set("Ошибка парсинга JSON".to_string());

                                        push_toast(
                                            toasts,
                                            "Ошибка парсинга JSON",
                                            ToastType::Error,
                                        );
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            error_msg.set("Ошибка чтения файла".to_string());

                            push_toast(toasts, "Ошибка чтения файла", ToastType::Error);
                        }
                    }

                    input.set_value("");
                });
            }
        }
    };

    let on_export = move |_| {
        let title = task_title.get();
        let desc = task_desc.get();

        simulator.with(|opt| {
            if let Some(sim) = opt {
                let state = sim.state.clone();
                let export_obj = serde_json::json!({
                    "metadata": {
                        "title": title,
                        "description": desc
                    },
                    "environment": { "threads": state.threads.len() },
                    "initial_state": state
                });

                let json_str = serde_json::to_string_pretty(&export_obj).unwrap();
                download_json(&json_str, "custom_scenario.json");
            };
        });
    };

    let on_strategy_change = move |event: Event| {
        let target = event
            .target()
            .unwrap()
            .dyn_into::<HtmlSelectElement>()
            .unwrap();
        let value = target.value();

        simulator.update(|opt| {
            if let Some(sim) = opt {
                sim.state.strategy = match value.as_str() {
                    "PythonGil" => Strategy::PythonGil,
                    "GoChannels" => Strategy::GoChannels,
                    _ => Strategy::default(),
                }
            };
        });
    };

    let on_generate = move |_| {
        let n = gen_count.get();
        let template = gen_template.get();
        let is_auto = is_auto_playing.clone();
        let sim = simulator.clone();

        let mut resources = Vec::new();
        let mut threads = Vec::new();

        if template == "philosophers" {
            task_title.set(format!("Обедающие философы ({} чел.)", n));
            task_desc.set("Классическая задача с круговым ожиданием.".to_string());

            for i in 1..=n {
                resources.push(create_mutex(i));

                let left_fork = i.to_string();
                let right_fork = (if i == n { 1 } else { i + 1 }).to_string();
                let (first_fork, second_fork) = if i == n {
                    (right_fork, left_fork)
                } else {
                    (left_fork, right_fork)
                };

                threads.push(create_thread(
                    i,
                    vec![
                        create_step("compute", None, 1),
                        create_step("lock", Some(&first_fork), 1),
                        create_step("lock", Some(&second_fork), 1),
                        create_step("compute", None, 3),
                        create_step("unlock", Some(&second_fork), 1),
                        create_step("unlock", Some(&first_fork), 1),
                    ],
                ));
            }
        } else if template == "race" {
            task_title.set(format!("Состояние гонки ({} потоков)", n));
            task_desc.set("Массовая конкуренция за один Mutex.".to_string());

            resources.push(create_mutex(1));

            for i in 1..=n {
                threads.push(create_thread(
                    i,
                    vec![
                        create_step("lock", Some("1"), 1),
                        create_step("compute", None, 2),
                        create_step("unlock", Some("1"), 1),
                    ],
                ));
            }
        } else if template == "producer" {
            task_title.set(format!("Производитель-потребитель ({} потоков)", n));
            task_desc.set("Паттерн взаимодействия через ограниченный буфер (Семафор).".to_string());

            resources.push(create_semaphore(1, 3));

            threads.push(create_thread(
                1,
                vec![
                    create_step("compute", None, 2),
                    create_step("lock", Some("1"), 1),
                    create_step("unlock", Some("1"), 1),
                ],
            ));

            for i in 2..=n {
                threads.push(create_thread(
                    i,
                    vec![
                        create_step("lock", Some("1"), 1),
                        create_step("compute", None, 4),
                        create_step("unlock", Some("1"), 1),
                    ],
                ));
            }
        } else if template == "readers" {
            task_title.set(format!("Читатели и писатели ({} потоков)", n));
            task_desc.set("Конкурентный доступ к БД (Семафор).".to_string());

            let cap = std::cmp::max(2, n / 2) as usize;
            resources.push(create_semaphore(1, cap));

            threads.push(create_thread(
                1,
                vec![
                    create_step("compute", None, 2),
                    create_step("lock", Some("1"), 1),
                    create_step("compute", None, 4),
                    create_step("unlock", Some("1"), 1),
                ],
            ));

            for i in 2..=n {
                threads.push(create_thread(
                    i,
                    vec![
                        create_step("lock", Some("1"), 1),
                        create_step("compute", None, 2),
                        create_step("unlock", Some("1"), 1),
                    ],
                ));
            }
        }

        is_auto.set(false);
        error_msg.set(String::new());
        sim.set(Some(Simulator::new(threads, resources)));

        let template_name = match template.as_str() {
            "philosophers" => "Обедающие философы",
            "race" => "Состояние гонки",
            "producer" => "Производитель-потребитель",
            "readers" => "Читатели и писатели",
            _ => "Сценарий",
        };

        push_toast(
            toasts,
            format!("Сгенерирован: {} ({} потоков)", template_name, n),
            ToastType::Info,
        );
    };

    view! {
        <div>
            <p class="panel-title">"Стратегия"</p>

            <select
                 class="task-btn"
                 style="width: 100%; margin-bottom: 15px; background: var(--bg-card);"
                 on:change=on_strategy_change
                 prop:value=move || simulator.with(|opt| {
                     opt.as_ref().map(|s| match s.state.strategy {
                        Strategy::CPthreads => "CPthreads",
                        Strategy::PythonGil => "PythonGil",
                        Strategy::GoChannels => "GoChannels",
                      }).unwrap_or("CPthreads")
                 })
                 prop:disabled=move || is_auto_playing.get()
            >
                <option value="CPthreads">"C (Pthreads)"</option>
                <option value="PythonGil">"Python (GIL)"</option>
                <option value="GoChannels">"Go (Channels)"</option>
            </select>

            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                <span style="font-size: 0.8rem; color: var(--muted);">"Порог голодания:"</span>
                <span style="font-size: 0.8rem; font-weight: bold; color: var(--text);">
                    {move || simulator.with(|opt| opt.as_ref().map(|s| s.state.starvation_threshold).unwrap_or(20))}
                </span>
            </div>

            <input type="range" min="10" max="100" step="5"
                prop:value={move || simulator.with(|opt| opt.as_ref().map(|s| s.state.starvation_threshold).unwrap_or(20).to_string())}
                on:input={move |ev| {
                    let val = ev.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()).map(|i| i.value()).unwrap_or_default();
                    if let Ok(v) = val.parse::<u64>() {
                       simulator.update(|opt| {
                        if let Some(sim) = opt {
                            sim.state.starvation_threshold = v;
                        }
                    });
                    }
                }}
                style="width: 100%; margin-bottom: 15px;"
            />

            {move || {
                   simulator.with(|opt| {
                     opt.as_ref().map(|sim| {
                           let total = sim.state.threads.len();
                           let completed = sim.state.threads.iter().filter(|t| t.status == simulator_core::ThreadStatus::Terminated).count();
                           let pct = if total > 0 {
                                (completed as f32 / total as f32 * 100.0) as u32
                           } else {
                                0
                           };

                           view! {
                               <div style="margin-bottom: 15px;">
                                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                        <span style="font-size: 0.8rem; color: var(--muted);">"Прогресс симуляции:"</span>
                                        <span style="font-size: 0.8rem; font-weight: bold; color: var(--text);">{format!("{}/{}", completed, total)}</span>
                                    </div>

                                    <div class="progress-bar">
                                        <div class="fill" style={format!("width: {}%", pct)}></div>
                                    </div>
                               </div>
                           }
                       })
                 })
            }}

            <p class="panel-title">"Генератор задач"</p>
            <select class="task-btn" style="width: 100%; margin-bottom: 8px; background: var(--bg-card);"
                on:change=move |ev| {
                let val = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok()).map(|s| s.value()).unwrap_or_default();
                gen_template.set(val)
            }
                prop:disabled=move || simulator.with(|s| s.is_some())>
                <option value="philosophers">"Обедающие философы"</option>
                <option value="race">"Гонка за ресурс"</option>
                <option value="producer">"Производитель-потребитель"</option>
                <option value="readers">"Читатели и писатели"</option>
            </select>

            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                <span style="font-size: 0.8rem; color: var(--muted);">"Количество (N):"</span>
                <span style="font-size: 0.8rem; font-weight: bold; color: var(--text);">{move || gen_count.get()}</span>
            </div>
            <input type="range" min="2" max="15" step="1"
                prop:value={move || gen_count.get().to_string()}
                on:input={move |ev| {
                let val = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()).map(|i| i.value()).unwrap_or_default();
                if let Ok(v) = val.parse::<u32>() { gen_count.set(v); }
            }}
                prop:disabled=move || simulator.with(|s| s.is_some())
                style="width: 100%; margin-bottom: 12px;"
            />

            <button class="task-btn" on:click=on_generate
                prop:disabled=move || simulator.with(|s| s.is_some())
                style="display: block; text-align: center; background: var(--success); color: white; margin-bottom: 20px; border-color: var(--success);">
                "Сгенерировать"
            </button>
            <hr style="border: 0; border-top: 1px solid var(--border); margin-bottom: 15px;" />

            <p class="panel-title">"Импорт"</p>
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! { <p style="color: var(--danger); font-size: 0.8rem; margin-bottom: 8px;">{err}</p> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
            <input
                type="file"
                id="file-upload"
                accept=".json"
                style="display: none;"
                on:change=on_file_change
            />
            <label
                for="file-upload"
                class="task-btn"
                style="display: block; text-align: center; background: var(--accent); color: white; margin-bottom: 15px; border-color: var(--accent);"
            >
                "Импорт локального файла"
            </label>

            <button
                class="task-btn"
                on:click=on_export
                prop:disabled=move || simulator.with(|s| s.is_none())
                style="display: block; text-align: center; margin-bottom: 15px;"
            >
                "Экспорт конфигурации"
            </button>

            <p class="panel-title">"Сценарии"</p>
            <ul class="task-list">
                {move || {
                    tasks.get().and_then(|list| list).unwrap_or_default().into_iter().map(|id| {
                         let id_btn = id.clone();
                           let load = load_task.clone();

                     view! {
                          <li>
                                <button class="task-btn" on:click=move |_| load(id_btn.clone())>
                                   {id.clone()}
                               </button>
                            </li>
                      }
                     }).collect::<Vec<_>>()
                }}
            </ul>
        </div>
    }
}
