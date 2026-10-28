//! Option Greeks Calculator (Phase 3.4b) at `/equity/option-greeks`.
//!
//! Live Black-Scholes-Merton pricing: every input change recomputes the
//! theoretical price and Greeks. A market price fills in an implied-vol solve.
//! Inputs are user-entered; no feed is wired, so the defaults are illustrative.

use super::black_scholes::{Greeks, bs_price_and_greeks, implied_vol};
use honba_kao_ui_core::{
    HfInput, HfKpi, HfPageHeader, HfPanel, HfPanelHeader, HfToggleGroup, HfToggleGroupItem,
};
use leptos::prelude::*;

const DEFAULT_SPOT: &str = "24850";
const DEFAULT_STRIKE: &str = "24850";
const DEFAULT_DAYS: &str = "30";
const DEFAULT_VOL_PCT: &str = "18";
const DEFAULT_RATE_PCT: &str = "6.5";

/// Parse a numeric field; `None` for blank, malformed or non-finite text.
fn parse_field(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Validated, unit-converted inputs ready for the pricing model.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Params {
    spot: f64,
    strike: f64,
    time_years: f64,
    vol: f64,
    rate: f64,
}

#[component]
fn GreeksField(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="greeks__field">
            <span class="kpi__label">{label}</span>
            {children()}
        </div>
    }
}

#[component]
pub fn OptionGreeksPage() -> impl IntoView {
    let spot = RwSignal::new(DEFAULT_SPOT.to_string());
    let strike = RwSignal::new(DEFAULT_STRIKE.to_string());
    let days = RwSignal::new(DEFAULT_DAYS.to_string());
    let vol_pct = RwSignal::new(DEFAULT_VOL_PCT.to_string());
    let rate_pct = RwSignal::new(DEFAULT_RATE_PCT.to_string());
    let market_price = RwSignal::new(String::new());
    let is_call = RwSignal::new(true);

    let params = Memo::new(move |_| {
        let spot = parse_field(&spot.get())?;
        let strike = parse_field(&strike.get())?;
        let days = parse_field(&days.get())?;
        let vol = parse_field(&vol_pct.get())? / 100.0;
        let rate = parse_field(&rate_pct.get())? / 100.0;
        if spot <= 0.0 || strike <= 0.0 || days <= 0.0 || vol <= 0.0 {
            return None;
        }
        Some(Params {
            spot,
            strike,
            time_years: days / 365.0,
            vol,
            rate,
        })
    });

    let greeks = Memo::new(move |_| -> Option<Greeks> {
        let p = params.get()?;
        Some(bs_price_and_greeks(
            p.spot,
            p.strike,
            p.time_years,
            p.vol,
            p.rate,
            is_call.get(),
        ))
    });

    let implied = Memo::new(move |_| -> Option<f64> {
        let p = params.get()?;
        let target = parse_field(&market_price.get()).filter(|v| *v > 0.0)?;
        let iv = implied_vol(
            target,
            p.spot,
            p.strike,
            p.time_years,
            p.rate,
            is_call.get(),
        );
        iv.is_finite().then_some(iv)
    });

    let fmt_or_dash = |value: Option<f64>, decimals: usize| match value {
        Some(v) => format!("{v:.decimals$}"),
        None => "—".to_string(),
    };

    view! {
        <div class="console fno greeks">
            <HfPageHeader title="Option Greeks Calculator" subtitle="Black-Scholes-Merton · live" />

            <HfPanel class="greeks__inputs">
                <HfPanelHeader title="Inputs" />
                <div class="greeks__form">
                    <GreeksField label="Spot">
                        <HfInput
                            value=DEFAULT_SPOT.to_string()
                            on_input=Callback::new(move |ev: web_sys::Event| spot.set(event_target_value(&ev)))
                        />
                    </GreeksField>
                    <GreeksField label="Strike">
                        <HfInput
                            value=DEFAULT_STRIKE.to_string()
                            on_input=Callback::new(move |ev: web_sys::Event| strike.set(event_target_value(&ev)))
                        />
                    </GreeksField>
                    <GreeksField label="Days to Expiry">
                        <HfInput
                            value=DEFAULT_DAYS.to_string()
                            on_input=Callback::new(move |ev: web_sys::Event| days.set(event_target_value(&ev)))
                        />
                    </GreeksField>
                    <GreeksField label="Volatility %">
                        <HfInput
                            value=DEFAULT_VOL_PCT.to_string()
                            on_input=Callback::new(move |ev: web_sys::Event| vol_pct.set(event_target_value(&ev)))
                        />
                    </GreeksField>
                    <GreeksField label="Risk-free Rate %">
                        <HfInput
                            value=DEFAULT_RATE_PCT.to_string()
                            on_input=Callback::new(move |ev: web_sys::Event| rate_pct.set(event_target_value(&ev)))
                        />
                    </GreeksField>
                    <GreeksField label="Market price">
                        <HfInput
                            placeholder="Optional, for implied vol"
                            on_input=Callback::new(move |ev: web_sys::Event| market_price.set(event_target_value(&ev)))
                        />
                        <span class="greeks__iv tnum">
                            {move || match implied.get() {
                                Some(iv) => format!("Implied Vol: {:.2}%", iv * 100.0),
                                None => "Implied Vol: —".to_string(),
                            }}
                        </span>
                    </GreeksField>
                </div>
                <div class="fno__toolbar">
                    <HfToggleGroup default_value="call">
                        <HfToggleGroupItem value="call" on_select=Callback::new(move |_| is_call.set(true))>
                            "Call"
                        </HfToggleGroupItem>
                        <HfToggleGroupItem value="put" on_select=Callback::new(move |_| is_call.set(false))>
                            "Put"
                        </HfToggleGroupItem>
                    </HfToggleGroup>
                    <span class="placeholder__note">"Theta per day · Vega per vol point · Rho per rate point"</span>
                </div>
            </HfPanel>

            <HfPanel class="greeks__outputs">
                <HfPanelHeader title="Theoretical Value & Greeks" />
                <div class="fno__kpis">
                    <HfKpi label="Price">
                        {move || fmt_or_dash(greeks.get().map(|g| g.price), 2)}
                    </HfKpi>
                    <HfKpi label="Delta">
                        {move || fmt_or_dash(greeks.get().map(|g| g.delta), 4)}
                    </HfKpi>
                    <HfKpi label="Gamma">
                        {move || fmt_or_dash(greeks.get().map(|g| g.gamma), 6)}
                    </HfKpi>
                    <HfKpi label="Theta (per day)">
                        {move || fmt_or_dash(greeks.get().map(|g| g.theta), 2)}
                    </HfKpi>
                    <HfKpi label="Vega (per 1%)">
                        {move || fmt_or_dash(greeks.get().map(|g| g.vega), 2)}
                    </HfKpi>
                    <HfKpi label="Rho (per 1%)">
                        {move || fmt_or_dash(greeks.get().map(|g| g.rho), 2)}
                    </HfKpi>
                </div>
            </HfPanel>
        </div>
    }
}
