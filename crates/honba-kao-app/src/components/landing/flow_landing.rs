//! The public landing flow. Mirrors `web/src/components/landing/FlowLanding.tsx`:
//! hero + product preview, supported-markets strip, three capability cards and
//! the "run it yourself" band.
//!
//! Structurally the reference layout, but every colour is a theme token and the
//! copy is honba-kao's: Indian exchanges and brokers instead of crypto venues.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::BrandLogo;
use crate::constants::branding;
use crate::router::paths::ROUTES;

#[component]
pub fn FlowLanding() -> impl IntoView {
    view! {
        <div class="flow-landing">
            <FlowHero />
            <FlowExchanges />
            <FlowFeatures />
            <FlowLocal />
        </div>
    }
}

#[component]
fn FlowHero() -> impl IntoView {
    view! {
        <section class="flow-hero">
            <div class="flow-hero-copy">
                <span class="flow-eyebrow">
                    <span class="flow-eyebrow__dot" />
                    "Open source. Self hosted."
                </span>

                <h1>
                    "Intelligence"
                    <br />
                    <em>"that trades NSE."</em>
                </h1>

                <p>"An open trading system you run yourself — built for Indian brokers."</p>

                <div class="flow-hero-actions">
                    <span class="flow-primary">
                        <A href=ROUTES::PORTFOLIO>
                            "Open your portfolio"
                            <svg
                                width="18"
                                height="18"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                aria-hidden="true"
                            >
                                <path d="M5 12h14M13 5l7 7-7 7" />
                            </svg>
                        </A>
                    </span>

                    <span class="flow-secondary">
                        <A href=ROUTES::REGISTER>
                            "Set up your workspace"
                            <svg
                                width="16"
                                height="16"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                aria-hidden="true"
                            >
                                <path d="M7 17 17 7M8 7h9v9" />
                            </svg>
                        </A>
                    </span>
                </div>

                <div class="flow-hero-trust">
                    <span>
                        <CheckIcon />
                        "Runs on your machine"
                    </span>
                    <span>
                        <CheckIcon />
                        "Your broker accounts"
                    </span>
                </div>
            </div>

            <div
                class="flow-product-preview"
                aria-label="Illustration of the Frontier planning workspace"
            >
                <div class="flow-preview-header">
                    <BrandLogo size=26 />
                    <span>"Frontier"</span>
                    <small>"Workspace preview"</small>
                </div>

                <div class="flow-preview-content">
                    <div class="flow-preview-market">
                        <span>"One idea. A clear plan."</span>
                        <b>"Compare your entry."</b>
                    </div>

                    <svg
                        class="flow-preview-chart"
                        viewBox="0 0 600 260"
                        role="img"
                        aria-label="Illustrative chart showing an entry, target and stop level"
                    >
                        <defs>
                            <linearGradient id="flow-chart-fill" x1="0" y1="0" x2="0" y2="1">
                                <stop
                                    offset="0"
                                    style="stop-color: var(--primary-8); stop-opacity: 0.16"
                                />
                                <stop
                                    offset="1"
                                    style="stop-color: var(--primary-8); stop-opacity: 0"
                                />
                            </linearGradient>
                        </defs>

                        <path d="M0 50H600" class="flow-chart-grid" />
                        <path d="M0 100H600" class="flow-chart-grid" />
                        <path d="M0 150H600" class="flow-chart-grid" />
                        <path d="M0 200H600" class="flow-chart-grid" />
                        <path d="M80 0V260" class="flow-chart-grid" />
                        <path d="M180 0V260" class="flow-chart-grid" />
                        <path d="M280 0V260" class="flow-chart-grid" />
                        <path d="M380 0V260" class="flow-chart-grid" />
                        <path d="M480 0V260" class="flow-chart-grid" />

                        <path
                            d="M0 214 30 200 52 210 80 185 105 190 133 156 154 169 182 128 203 137 229 98 250 119 274 108 300 145 326 129 352 142 378 100 398 115 425 77 449 94 471 59 494 81 520 66V260H0Z"
                            fill="url(#flow-chart-fill)"
                        />
                        <path
                            d="m0 214 30-14 22 10 28-25 25 5 28-34 21 13 28-41 21 9 26-39 21 21 24-11 26 37 26-16 26 13 26-42 20 15 27-38 24 17 22-35 23 22 26-15"
                            class="flow-chart-line"
                        />

                        <rect x="350" y="48" width="174" height="87" class="flow-zone-target" />
                        <rect x="350" y="135" width="174" height="45" class="flow-zone-stop" />

                        <path d="M350 48H600" class="flow-level-target" />
                        <text x="536" y="39" class="flow-label-target">"Target"</text>

                        <path d="M350 135H600" class="flow-level-entry" />
                        <text x="536" y="126" class="flow-label-entry">"Entry"</text>

                        <path d="M350 180H600" class="flow-level-stop" />
                        <text x="536" y="198" class="flow-label-stop">"Stop"</text>
                    </svg>

                    <div class="flow-preview-plans">
                        <div>
                            <span>"01"</span>
                            <strong>"Market"</strong>
                            <small>"Enter now"</small>
                        </div>
                        <div data-selected="true">
                            <span>"02"</span>
                            <strong>"Limit"</strong>
                            <small>"Choose a price"</small>
                        </div>
                        <div>
                            <span>"03"</span>
                            <strong>"Ladder"</strong>
                            <small>"Split by price"</small>
                        </div>
                        <div>
                            <span>"04"</span>
                            <strong>"TWAP"</strong>
                            <small>"Split by time"</small>
                        </div>
                    </div>

                    <div class="flow-preview-bottom">
                        <span>
                            <CheckIcon />
                            "Review before you trade"
                        </span>
                        <span>
                            "Make your next move"
                            <svg
                                width="14"
                                height="14"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                aria-hidden="true"
                            >
                                <path d="M7 17 17 7M8 7h9v9" />
                            </svg>
                        </span>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn FlowExchanges() -> impl IntoView {
    view! {
        <section class="flow-exchanges" aria-label="Supported markets and brokers">
            <span>"Your markets. Connected."</span>
            <b>"NSE"</b>
            <b>"BSE"</b>
            <b>"Zerodha"</b>
            <b>"Dhan"</b>
        </section>
    }
}

#[component]
fn FlowFeatures() -> impl IntoView {
    view! {
        <section class="flow-features">
            <div class="flow-section-heading">
                <span class="flow-eyebrow">"One connected workspace"</span>
                <h2>
                    "Less friction."
                    <br />
                    "More clarity."
                </h2>
                <p>"AI helps you explore. Clear controls help you act."</p>
            </div>

            <article>
                <LayersIcon />
                <h3>"See the alternatives."</h3>
                <p>
                    "Compare market, limit, ladder and TWAP plans. See entries, size and exits before committing."
                </p>
                <A href=ROUTES::PORTFOLIO>
                    "One portfolio for everything"
                    <ArrowUpRight />
                </A>
            </article>

            <article>
                <WalletIcon />
                <h3>"Know where you stand."</h3>
                <p>
                    "Every broker in one book. Holdings, funds and margin stay in view — Zerodha, Dhan, Upstox, Angel One, Fyers and Groww."
                </p>
                <A href=ROUTES::PORTFOLIO>
                    "Connect your accounts"
                    <ArrowUpRight />
                </A>
            </article>

            <article>
                <SlidersIcon />
                <h3>"Make it your own."</h3>
                <p>
                    "Choose your AI model and broker accounts. Set risk gates once, then let agents trade within them."
                </p>
                <A href=ROUTES::STRATEGY>
                    "Build a strategy"
                    <ArrowUpRight />
                </A>
            </article>
        </section>
    }
}

#[component]
fn FlowLocal() -> impl IntoView {
    view! {
        <section class="flow-local">
            <div>
                <span class="flow-eyebrow">
                    <TerminalIcon />
                    "Built to run locally"
                </span>
                <h2>
                    "Your workspace."
                    <br />
                    "On your terms."
                </h2>
                <p>
                    "Install honba-kao on your machine, connect your broker accounts and choose how you want to trade."
                </p>
            </div>

            <a href=branding::REPO_URL target="_blank" rel="noopener noreferrer">
                "Installation guide"
                <svg
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                >
                    <path d="M7 17 17 7M8 7h9v9" />
                </svg>
            </a>
        </section>
    }
}

#[component]
fn CheckIcon() -> impl IntoView {
    view! {
        <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="m20 6-11 11-5-5" />
        </svg>
    }
}

#[component]
fn ArrowUpRight() -> impl IntoView {
    view! {
        <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M7 17 17 7M8 7h9v9" />
        </svg>
    }
}

#[component]
fn LayersIcon() -> impl IntoView {
    view! {
        <svg
            width="23"
            height="23"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z" />
            <path d="m22 17.65-9.17 4.16a2 2 0 0 1-1.66 0L2 17.65" />
            <path d="m22 12.65-9.17 4.16a2 2 0 0 1-1.66 0L2 12.65" />
        </svg>
    }
}

#[component]
fn WalletIcon() -> impl IntoView {
    view! {
        <svg
            width="23"
            height="23"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M19 7V4a1 1 0 0 0-1-1H5a2 2 0 0 0 0 4h15a1 1 0 0 1 1 1v4h-3a2 2 0 0 0 0 4h3a1 1 0 0 0 1-1v-2" />
            <path d="M3 5v14a2 2 0 0 0 2 2h15a1 1 0 0 0 1-1v-4" />
        </svg>
    }
}

#[component]
fn SlidersIcon() -> impl IntoView {
    view! {
        <svg
            width="23"
            height="23"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M21 4h-7M10 4H3M21 12h-9M8 12H3M21 20h-5M12 20H3" />
            <path d="M14 2v4M8 10v4M16 18v4" />
        </svg>
    }
}

#[component]
fn TerminalIcon() -> impl IntoView {
    view! {
        <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="m4 17 6-6-6-6" />
            <path d="M12 19h8" />
        </svg>
    }
}
