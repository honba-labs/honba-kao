use leptos::prelude::*;

#[component]
pub fn HfTerminalShell(
    header: AnyView,
    nav_rail: AnyView,
    content: AnyView,
    footer: AnyView,
    #[prop(into, optional)] overlay: Option<AnyView>,
) -> impl IntoView {
    view! {
        <div class="hf-terminal-shell flex flex-col h-screen w-screen overflow-hidden bg-surface-base text-text-high">
            <header class="hf-shell-header shrink-0 border-b border-border-soft">
                {header}
            </header>
            <div class="hf-shell-main flex flex-grow overflow-hidden relative">
                <aside class="hf-shell-rail shrink-0 border-r border-border-soft">
                    {nav_rail}
                </aside>
                <main class="hf-shell-viewport flex-grow overflow-y-auto min-w-0">
                    {content}
                </main>
                {overlay.map(|ov| view! { <div class="hf-shell-overlay">{ov}</div> })}
            </div>
            <footer class="hf-shell-footer shrink-0 border-t border-border-soft">
                {footer}
            </footer>
        </div>
    }
}
