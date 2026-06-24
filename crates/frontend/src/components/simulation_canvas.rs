use leptos::prelude::*;

use crate::utils::generate_svg;

#[component]
pub fn SimulationCanvas() -> impl IntoView {
    let simulator = use_context::<RwSignal<Option<simulator_core::Simulator>>>()
        .expect("Контекст симулятора не найден");

    let svg_content = move || {
        simulator.with(|opt| {
            opt.as_ref()
                .map(|sim| generate_svg(sim))
                .unwrap_or_else(|| {
                    "<svg width=\"600\" height=\"400\" xmlns=\"http://www.w3.org/2000/svg\">\
	                    <rect width=\"100%\" height=\"100%\" fill=\"#f8f9fa\"/>\
	                    <text x=\"50%\" y=\"50%\" text-anchor=\"middle\" dy=\".3em\" \
	                        fill=\"#6c757d\" font-size=\"16\">\
	                    Нет данных для визуализации\
	                    </text>
	                </svg>"
                        .to_string()
                })
        })
    };

    view! {
        <div class="simulation-canvas">
            <div class="canvas-panel panel-card">
                <p class="panel-title">"Граф ресурсов и потоков"</p>
                <div class="svg-wrap" inner_html=svg_content></div>

                <div class="legend">
                    <div class="legend-item">
                        <span class="leg-dot" style="background: #2563eb;"></span>
                        <span>"Выполняется"</span>
                    </div>
                    <div class="legend-item">
                        <span class="leg-dot" style="background: #16a34a;"></span>
                        <span>"Готов"</span>
                    </div>
                    <div class="legend-item">
                        <span class="leg-dot" style="background: #dc2626;"></span>
                        <span>"Заблокирован"</span>
                    </div>
                    <div class="legend-item">
                        <span class="leg-dot" style="background: #94a3b8;"></span>
                        <span>"Завершен"</span>
                    </div>
                    <div class="legend-item">
                        <span class="leg-dot" style="background: #7c3aed;"></span>
                        <span>"Новый"</span>
                    </div>
                    <div class="legend-item">
                        <span class="leg-line" style="background: #22c55e;"></span>
                        <span>"Ресурс занят"</span>
                    </div>
                    <div class="legend-item">
                        <span class="leg-line-dashed"></span>
                        <span>"Ожидание ресурса"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
