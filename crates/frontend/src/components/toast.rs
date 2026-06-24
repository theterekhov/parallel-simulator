use leptos::prelude::*;

use crate::models::toast::ToastType;

#[component]
pub fn ToastContainer() -> impl IntoView {
    let toasts = use_context::<RwSignal<Vec<crate::models::toast::Toast>>>()
        .expect("Контекст toasts не найден");

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
                        view! { <div class=toast_class>{toast.message}</div> }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}
