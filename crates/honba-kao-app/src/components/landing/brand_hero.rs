//! Hero section. Mirrors `web/src/components/landing/brand/BrandHero.tsx`.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::constants::branding;
use crate::router::paths::ROUTES;
use crate::utils::{compact_currency, currency};

#[component]
pub fn BrandHero() -> impl IntoView {
    view! {
        <section class="hero">
            <div class="hero__copy">
                <span class="hero__eyebrow">"AI agents · NSE & BSE · paper first"</span>
                <h1 class="hero__title">
                    "Describe the trade. "
                    <em>"The agent plans it."</em>
                </h1>
                <p class="hero__sub">{branding::TAGLINE}</p>

                <div class="hero__cta">
                    <A href=ROUTES::REGISTER>"Start paper trading"</A>
                    <A href=ROUTES::LOGIN>"Sign in"</A>
                </div>
            </div>

            <div class="hero__terminal">
                <div class="hero__terminal-head">
                    <span class="dot dot--red" />
                    <span class="dot dot--amber" />
                    <span class="dot dot--green" />
                    <span class="hero__terminal-title">"nifty-momentum · live"</span>
                </div>
                <div class="hero__terminal-body">
                    <div class="hero__row">
                        <span>"Total value"</span>
                        <strong class="tnum">{compact_currency(1_842_650.0)}</strong>
                    </div>
                    <div class="hero__row">
                        <span>"Day P&L"</span>
                        <strong class="tnum pnl-up">{currency(14_850.0)}</strong>
                    </div>
                    <div class="hero__row">
                        <span>"RELIANCE"</span>
                        <strong class="tnum">{currency(2884.20)}</strong>
                    </div>
                    <div class="hero__log">
                        <p><span class="mono">"15:24:02"</span>" gate PASSED · BUY 50 RELIANCE"</p>
                        <p><span class="mono">"15:23:45"</span>" gate REJECTED · drawdown limit"</p>
                        <p><span class="mono">"15:21:10"</span>" gate REJECTED · circuit filter"</p>
                    </div>
                </div>
            </div>
        </section>
    }
}
