use honba_kao_ui_core::primitive::{ButtonVariant, HfButton};
use leptos::prelude::*;

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-surface-base text-text-primary flex flex-col justify-between">
            <header class="h-16 border-b border-border-soft px-8 flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded bg-primary-600 flex items-center justify-center font-mono font-bold text-white text-sm">
                        "HK"
                    </div>
                    <span class="font-bold tracking-tight text-lg">"HONBA-KAO"</span>
                    <span class="text-[10px] uppercase font-mono px-2 py-0.5 rounded bg-surface-subtle text-text-muted border border-border-soft">
                        "v3.0 Master"
                    </span>
                </div>
                <div class="flex items-center gap-4">
                    <a href="/auth">
                        <HfButton variant=ButtonVariant::Ghost>
                            "Sign In"
                        </HfButton>
                    </a>
                    <a href="/trading/cockpit">
                        <HfButton variant=ButtonVariant::Primary>
                            "Enter Terminal"
                        </HfButton>
                    </a>
                </div>
            </header>

            <main class="flex-1 max-w-5xl mx-auto px-6 py-20 flex flex-col items-center text-center justify-center">
                <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-primary-500/30 bg-primary-500/10 text-primary-400 text-xs font-mono mb-6">
                    "INSTITUTIONAL GRADE • ZERO COMPROMISE"
                </div>
                <h1 class="text-4xl sm:text-6xl font-extrabold tracking-tight mb-6">
                    "Ultra-Fast Financial Architecture"
                </h1>
                <p class="text-base sm:text-lg text-text-muted max-w-2xl mb-10">
                    "Unified trading, derivatives analytics, portfolio margin intelligence, and quantitative simulation engine built on WebAssembly and Rust."
                </p>
                <div class="flex items-center gap-4">
                    <a href="/trading/cockpit">
                        <HfButton variant=ButtonVariant::Primary class="px-6 py-3 text-sm">
                            "Launch Terminal Now"
                        </HfButton>
                    </a>
                    <a href="/research/markets">
                        <HfButton variant=ButtonVariant::Secondary class="px-6 py-3 text-sm">
                            "Explore Markets"
                        </HfButton>
                    </a>
                </div>
            </main>

            <footer class="border-t border-border-soft py-6 px-8 text-center text-xs text-text-muted font-mono">
                "Honba-Kao Unified Institutional Workstation • High-Precision WASM Math Engine"
            </footer>
        </div>
    }
}
