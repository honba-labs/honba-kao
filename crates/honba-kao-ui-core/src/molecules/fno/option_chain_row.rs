//! `HfOptionChainRow` — straddle row coupling CE and PE quotes around a strike.
//! Ported from the restructure spec (Section 7.4) and `secured-finance-app`.

use leptos::prelude::*;
use crate::primitive::{HfMoneynessBadge, Moneyness};

#[derive(Clone, Debug, PartialEq)]
pub struct OptionLegQuote {
    pub ltp: f64,
    pub change_pct: f64,
    pub volume: u64,
    pub open_interest: u64,
    pub oi_change: i64,
    pub iv: f64,
    pub delta: f64,
    pub gamma: f64,
}

fn oi_bar(oi: u64, oi_change: i64, max_oi: u64) -> impl IntoView {
    let pct = if max_oi > 0 { oi as f64 / max_oi as f64 * 100.0 } else { 0.0 };
    let change_class = if oi_change >= 0 { "text-emerald-400" } else { "text-rose-400" };
    view! {
        <div class="hf-oi-bar flex items-center gap-1 text-[10px]">
            <div class="hf-oi-fill bg-surface-elevated h-1.5 w-full rounded overflow-hidden">
                <div class="hf-oi-fill-inner bg-text-muted h-full" style=format!("width: {:.1}%;", pct.min(100.0)) />
            </div>
            <span class=change_class>{format!("{:+}", oi_change)}</span>
        </div>
    }
}

#[component]
pub fn HfOptionChainRow(
    strike: f64,
    spot_price: f64,
    call: OptionLegQuote,
    put: OptionLegQuote,
    #[prop(into)] on_select_leg: Callback<(f64, bool, bool)>, // (strike, is_call, is_buy)
    #[prop(into, optional)] max_oi: Option<u64>,
) -> impl IntoView {
    let max_oi = max_oi.unwrap_or(call.open_interest.max(put.open_interest).max(1));
    let is_call_itm = spot_price > strike;
    let is_put_itm = spot_price < strike;
    let is_atm = (spot_price - strike).abs() < (strike * 0.005);

    view! {
        <tr class=move || format!("hf-option-chain-row {}", if is_atm { "bg-surface-elevated font-semibold" } else { "" })>
            // CALL SIDE (Left)
            <td class=move || format!("{} p-1", if is_call_itm { "bg-emerald-950/20" } else { "" })>
                {format!("{:.2}", call.delta)}
            </td>
            <td class=move || format!("{} p-1", if is_call_itm { "bg-emerald-950/20" } else { "" })>
                {format!("{:.1}%", call.iv * 100.0)}
            </td>
            <td class="p-1">{oi_bar(call.open_interest, call.oi_change, max_oi)}</td>
            <td class="cursor-pointer hover:bg-emerald-500/10 p-1" on:click=move |_| on_select_leg.run((strike, true, true))>
                <span class=move || format!("{} font-mono", if call.change_pct >= 0.0 { "text-emerald-400" } else { "text-rose-400" })>
                    {format!("{:.2}", call.ltp)}
                </span>
            </td>

            // CENTER STRIKE
            <td class="hf-strike-cell text-center px-2 py-1 font-mono font-bold bg-surface-subtle border-x border-border-soft">
                <span>{format!("{:.0}", strike)}</span>
                {if is_atm {
                    view! { <HfMoneynessBadge moneyness=Moneyness::ATM /> }.into_any()
                } else if is_call_itm {
                    view! { <HfMoneynessBadge moneyness=Moneyness::ITM /> }.into_any()
                } else {
                    view! { <HfMoneynessBadge moneyness=Moneyness::OTM /> }.into_any()
                }}
            </td>

            // PUT SIDE (Right)
            <td class="cursor-pointer hover:bg-rose-500/10 p-1" on:click=move |_| on_select_leg.run((strike, false, true))>
                <span class=move || format!("{} font-mono", if put.change_pct >= 0.0 { "text-emerald-400" } else { "text-rose-400" })>
                    {format!("{:.2}", put.ltp)}
                </span>
            </td>
            <td class="p-1">{oi_bar(put.open_interest, put.oi_change, max_oi)}</td>
            <td class=move || format!("{} p-1", if is_put_itm { "bg-rose-950/20" } else { "" })>
                {format!("{:.1}%", put.iv * 100.0)}
            </td>
            <td class=move || format!("{} p-1", if is_put_itm { "bg-rose-950/20" } else { "" })>
                {format!("{:.2}", put.delta)}
            </td>
        </tr>
    }
}
