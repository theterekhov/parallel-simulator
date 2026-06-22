mod components;
mod models;
mod utils;

use gloo_net::http::Request;
use leptos::prelude::*;
use simulator_core::Simulator;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;

use crate::components::{
    config_panel::ConfigPanel, log_console::LogConsole, report::ReportModal,
    simulation_canvas::SimulationCanvas, toast::ToastContainer,
};
use crate::models::{SimulationReport, Toast, ToastType, generate_report, push_toast};
use crate::utils::generate_svg;

fn App() -> impl IntoView {
    let tasks: RwSignal<Vec<String>> = RwSignal::new(vec![]);
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

    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(resp) = Request::get("/api/tasks").send().await {
                if let Ok(list) = resp.json::<Vec<String>>().await {
                    tasks.set(list);
                }
            }
        });
    });

    let interval_id = RwSignal::new(None);

    Effect::new(move |prev| {
        let _ = prev;
        let playing = is_auto_playing.get();

        if let Some(id) = interval_id.get_untracked() {
            web_sys::window().unwrap().clear_interval_with_handle(id);
        }

        interval_id.set(None);

        if playing {
            let sim_sig = simulator;
            let play_sig = is_auto_playing;
            let id_ref = interval_id;
            let report_sig = show_report;
            let toast_sig = toasts;

            let tick = Closure::wrap(Box::new(move || {
                sim_sig.update(|opt| {
                    if let Some(sim) = opt {
                        if !sim.is_finished() {
                            sim.tick();
                        } else {
                            play_sig.set(false);
                            report_sig.set(Some(generate_report(sim)));

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

                            push_toast(toast_sig, msg, toast_type);
                        }
                    } else {
                        play_sig.set(false);
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
            id_ref.set(Some(id));
        }
    });

    let toggle_play = move || {
        is_auto_playing.update(|v| *v = !*v);
    };
    let do_step = move || {
        simulator.update(|opt| {
            if let Some(sim) = opt {
                if !sim.is_finished() {
                    sim.tick();
                }
            }
        });
    };
    let do_reset = move || {
        is_auto_playing.set(false);
        simulator.set(None);
    };

    Effect::new(move |_| {
        let window = web_sys::window().unwrap();
        let on_keydown = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            match event.key().as_str() {
                " " => {
                    event.prevent_default();

                    is_auto_playing.update(|v| *v = !*v);
                }
                "ArrowRight" => {
                    event.prevent_default();

                    simulator.update(|opt| {
                        if let Some(sim) = opt {
                            if !sim.is_finished() {
                                sim.tick();
                            }
                        }
                    });
                }
                "r" | "R" => {
                    event.prevent_default();

                    is_auto_playing.set(false);
                    simulator.set(None);
                }
                _ => {}
            }
        }) as Box<dyn FnMut(_)>);

        window
            .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
            .unwrap();

        on_cleanup(move || {
            window
                .remove_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
                .unwrap();
        });

        on_keydown.forget();
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
    let is_finished = move || simulator.with(|s| s.as_ref().map_or(false, |sim| sim.is_finished()));

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
                      "Шаг ->"
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
