use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[derive(Debug, Clone, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub toast_type: ToastType,
}

pub fn push_toast(toasts: RwSignal<Vec<Toast>>, message: impl Into<String>, toast_type: ToastType) {
    let id = web_sys::js_sys::Date::now() as u64;
    let toast = Toast {
        id,
        message: message.into(),
        toast_type,
    };

    toasts.update(|t| t.push(toast));

    let window = web_sys::window().unwrap();
    let closure = Closure::wrap(Box::new(move || {
        toasts.update(|t| t.retain(|toast| toast.id != id));
    }) as Box<dyn FnMut()>);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        4000,
    );
    closure.forget();
}

pub fn remove_toast(toasts: RwSignal<Vec<Toast>>, id: u64) {
    toasts.update(|t| t.retain(|toast| toast.id != id));
}
