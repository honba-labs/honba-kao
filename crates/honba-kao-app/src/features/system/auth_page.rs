use honba_kao_ui_core::primitive::{ButtonVariant, HfButton, HfInput};
use leptos::prelude::*;

#[component]
pub fn AuthPage() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let api_key = RwSignal::new(String::new());

    view! {
        <div class="min-h-screen bg-surface-base text-text-primary flex items-center justify-center p-6">
            <div class="w-full max-w-md p-8 rounded-lg border border-border-soft bg-surface-subtle shadow-lg">
                <div class="flex items-center gap-3 mb-6">
                    <div class="w-8 h-8 rounded bg-primary-600 flex items-center justify-center font-mono font-bold text-white text-sm">
                        "HK"
                    </div>
                    <div>
                        <h2 class="text-lg font-bold">"Institutional Access"</h2>
                        <p class="text-xs text-text-muted">"Sign in with broker credentials or Web3 wallet"</p>
                    </div>
                </div>

                <div class="flex flex-col gap-4">
                    <div>
                        <label class="text-xs text-text-muted mb-1 block">"Institutional Email / Trader ID"</label>
                        <HfInput value=email placeholder="trader@fund.com".to_string() />
                    </div>
                    <div>
                        <label class="text-xs text-text-muted mb-1 block">"Session Access Key"</label>
                        <HfInput value=api_key placeholder="hk_live_...".to_string() />
                    </div>

                    <a href="/trading/cockpit" class="mt-2">
                        <HfButton variant=ButtonVariant::Primary class="w-full font-semibold">
                            "Authenticate & Enter Terminal"
                        </HfButton>
                    </a>

                    <div class="relative my-2 text-center text-xs text-text-muted">
                        <span class="bg-surface-subtle px-2">"OR CONNECT SECURE ENCLAVE"</span>
                    </div>

                    <HfButton variant=ButtonVariant::Secondary class="w-full text-xs">
                        "Connect Web3 Wallet (Ledger / Safe / MetaMask)"
                    </HfButton>
                </div>
            </div>
        </div>
    }
}
