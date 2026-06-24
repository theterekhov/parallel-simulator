use leptos::reactive::{signal::RwSignal, traits::Update};

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
}

pub fn remove_toast(toasts: RwSignal<Vec<Toast>>, id: u64) {
    toasts.update(|t| t.retain(|toast| toast.id != id));
}
