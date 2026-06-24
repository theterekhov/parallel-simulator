use frontend::components::config_panel::ConfigPanel;
use frontend::components::log_console::LogConsole;
use frontend::components::report::ReportModal;
use frontend::components::simulation_canvas::SimulationCanvas;
use frontend::components::toast::ToastContainer;
use frontend::models::simulation_report::{SimulationReport, generate_report};
use frontend::models::toast::{Toast, ToastType, push_toast};
use gloo_net::http::Request;
use leptos::{ev, prelude::*};
use simulator_core::Simulator;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

#[component]
fn App() -> impl IntoView {
    let tasks = LocalResource::new(|| async move {
        Request::get("/api/tasks")
            .send()
            .await
            .ok()?
            .json::<Vec<String>>()
            .await
            .ok()
    });
    let simulator: RwSignal<Option<Simulator>> = RwSignal::new(None);
    let is_auto_playing: RwSignal<bool> = RwSignal::new(false);
    let show_report: RwSignal<Option<SimulationReport>> = RwSignal::new(None);
    let toasts: RwSignal<Vec<Toast>> = RwSignal::new(vec![]);

    let task_title: RwSignal<String> = RwSignal::new(String::new());
    let task_desc: RwSignal<String> = RwSignal::new(String::new());
    let error_msg: RwSignal<String> = RwSignal::new(String::new());

    provide_context(simulator);
    provide_context(is_auto_playing);
    provide_context(task_title);
    provide_context(task_desc);
    provide_context(show_report);
    provide_context(toasts);
    provide_context(error_msg);

    let interval_id = RwSignal::new(None);

    Effect::new(move || {
        let playing = is_auto_playing.get();

        if let Some(id) = interval_id.get_untracked() {
            web_sys::window().unwrap().clear_interval_with_handle(id);
        }

        interval_id.set(None);

        if playing {
            let tick = Closure::wrap(Box::new(move || {
                simulator.update(|opt| {
                    if let Some(sim) = opt {
                        if !sim.is_finished() {
                            sim.tick();
                        } else {
                            is_auto_playing.set(false);
                            show_report.set(Some(generate_report(sim)));

                            let msg = if sim.state.is_deadlocked {
                                "Обнаружена взаимная блокировка!"
                            } else {
                                "Симуляция успешно завершена"
                            };

                            let toast_type = if sim.state.is_deadlocked {
                                ToastType::Warning
                            } else {
                                ToastType::Success
                            };

                            push_toast(toasts, msg, toast_type);
                        }
                    } else {
                        is_auto_playing.set(false);
                    }
                });
            }) as Box<dyn FnMut()>);

            let window = web_sys::window().unwrap();
            let id = window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    tick.as_ref().unchecked_ref(),
                    800,
                )
                .unwrap();

            tick.forget();
            interval_id.set(Some(id));

            on_cleanup(move || {
                window.clear_interval_with_handle(id);
            });
        }
    });

    let toggle_play = move || {
        let finished = simulator.with(|opt| opt.as_ref().is_some_and(|s| s.is_finished()));
        if !finished {
            is_auto_playing.update(|v| *v = !*v);
        }
    };
    let do_step = move || {
        simulator.update(|opt| {
            if let Some(sim) = opt {
                if !sim.is_finished() {
                    sim.tick();
                    if sim.is_finished() {
                        show_report.set(Some(generate_report(sim)));
                        let msg = if sim.state.is_deadlocked {
                            "Обнаружена взаимная блокировка!"
                        } else {
                            "Симуляция успешно завершена"
                        };
                        let toast_type = if sim.state.is_deadlocked {
                            ToastType::Warning
                        } else {
                            ToastType::Success
                        };
                        push_toast(toasts, msg, toast_type);
                    }
                }
            }
        });
    };
    let do_reset = move || {
        is_auto_playing.set(false);
        simulator.set(None);
    };

    Effect::new(move |_| {
        let handle =
            window_event_listener(
                ev::keydown,
                move |event: web_sys::KeyboardEvent| match event.key().as_str() {
                    " " => {
                        event.prevent_default();
                        is_auto_playing.update(|v| *v = !*v);
                    }
                    "ArrowRight" => {
                        event.prevent_default();
                        do_step();
                    }
                    "r" | "R" => {
                        event.prevent_default();
                        is_auto_playing.set(false);
                        simulator.set(None);
                    }
                    _ => {}
                },
            );

        on_cleanup(move || handle.remove());
    });

    let play_label = move || {
        if is_auto_playing.get() {
            "Пауза"
        } else {
            "Старт"
        }
    };
    let can_play = move || simulator.with(|s| s.as_ref().map_or(false, |sim| !sim.is_finished()));
    let can_step = move || {
        !is_auto_playing.get()
            && simulator.with(|s| s.as_ref().map_or(false, |sim| !sim.is_finished()))
    };
    let can_reset = move || simulator.with(|s| s.is_some());

    view! {
        <div class="app-root">
            <ToastContainer />
            <ReportModal />

            <header class="app-header">

                <div>
                    <div class="app-title">"Симулятор параллельных вычислений"</div>
                    <div class="app-subtitle">{move || task_desc.get()}</div>
                </div>

                 <div class="btn-group">
                     <button
                         class={move || if is_auto_playing.get() {"btn-ctrl btn-pause"} else {"btn-ctrl btn-play"}}
                          on:click=move |_| toggle_play()
                           disabled=move || !can_play()
                      >
                      {play_label}
                      </button>
                      <button
                            class="btn-ctrl btn-step"
                            on:click=move |_| do_step()
                            disabled=move || !can_step()
                      >
                      "Шаг"
                      </button>
                      <button
                            class="btn-ctrl btn-reset"
                            on:click=move |_| do_reset()
                            disabled=move || !can_reset()
                      >
                      "Сброс"
                      </button>
                 </div>

            </header>


            <div class="app-body">
                <div class="config-col">
                    <ConfigPanel tasks=tasks />
                </div>
                <div class="main-col">
                    <SimulationCanvas />
                    <LogConsole />
                </div>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    leptos::mount::mount_to_body(App);
}
