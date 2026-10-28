use leptos::prelude::*;

#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="site-footer flex items-center justify-between px-4 py-1.5 bg-surface-subtle border-t border-border-soft text-[11px] text-text-muted font-mono select-none">
            <div class="flex items-center gap-3">
                <div class="flex items-center gap-1.5">
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                    <span>"OMS: CONNECTED"</span>
                </div>
                <span>"|"</span>
                <span>"LATENCY: 12ms"</span>
                <span>"|"</span>
                <span>"FEED: 100 ticks/s"</span>
            </div>
            <div>
                <span>"HONBA-KAO v3.0 Institutional"</span>
            </div>
        </footer>
    }
}
