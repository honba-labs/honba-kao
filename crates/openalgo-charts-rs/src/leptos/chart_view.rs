use crate::theme::ChartTheme;
use crate::types::CandleData;
use leptos::prelude::*;

#[component]
pub fn OpenalgoChart(
    #[prop(into)] data: Signal<Vec<CandleData>>,
    #[prop(into, optional)] _theme: Option<ChartTheme>,
    #[prop(into, optional)] height: Option<String>,
) -> impl IntoView {
    let container_ref = NodeRef::<leptos::html::Div>::new();
    let chart_height = height.unwrap_or_else(|| "100%".to_string());

    Effect::new(move |_| {
        let _candles = data.get();
        if let Some(_container) = container_ref.get() {
            // Mounted or updated canvas
        }
    });

    view! {
        <div
            node_ref=container_ref
            class="openalgo-chart-container relative w-full h-full min-h-[300px] bg-surface-base overflow-hidden"
            style=format!("height: {};", chart_height)
        >
            <div class="absolute inset-0 flex items-center justify-center text-xs text-text-muted pointer-events-none">
                {move || if data.get().is_empty() { "No Chart Data" } else { "" }}
            </div>
        </div>
    }
}
