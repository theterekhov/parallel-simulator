use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use crate::models::toast::{Toast, ToastType, remove_toast};

#[component]
pub fn ToastContainer() -> impl IntoView {
    let toasts = use_context::<RwSignal<Vec<Toast>>>().expect("Контекст toasts не найден");

    view! {
        <div class="toast-container">
            {move || {
                toasts
                    .get()
                    .into_iter()
                    .map(|toast| {
                        let toast_class = match toast.toast_type {
                            ToastType::Success => "toast success",
                            ToastType::Error => "toast error",
                            ToastType::Info => "toast info",
                            ToastType::Warning => "toast warning",
                        };
                        let id = toast.id;
                        let toasts_for_timer = toasts;
                        let window = web_sys::window().unwrap();
                        let closure = Closure::wrap(Box::new(move || {
                            remove_toast(toasts_for_timer, id);
                        }) as Box<dyn FnMut()>);
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            4000,
                        );
                        closure.forget();
                        view! { <div class=toast_class>{toast.message}</div> }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}
