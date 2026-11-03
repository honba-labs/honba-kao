use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct StatItem {
    pub label: String,
    pub value: String,
    pub is_positive: Option<bool>,
}

#[component]
pub fn HfStatsBar(#[prop(into)] stats: Signal<Vec<StatItem>>) -> impl IntoView {
    view! {
        <div class="hf-stats-bar flex items-center gap-4 px-3 py-1.5 bg-surface-subtle border-b border-border-soft text-xs overflow-x-auto">
            <For
                each=move || stats.get()
                key=|st| st.label.clone()
                children=move |st| {
                    let color = match st.is_positive {
                        Some(true) => "text-emerald-400 font-mono font-semibold",
                        Some(false) => "text-rose-400 font-mono font-semibold",
                        None => "text-text-high font-mono font-semibold",
                    };
                    view! {
                        <div class="flex items-center gap-1.5 shrink-0">
                            <span class="text-text-muted">{st.label}:</span>
                            <span class=color>{st.value}</span>
                        </div>
                    }
                }
            />
        </div>
    }
}
