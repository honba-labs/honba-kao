//! `HfOptionChainRow` — straddle row coupling CE and PE quotes around a strike.
//! Ported from the restructure spec (Section 7.4) and `secured-finance-app`.
//! All quote/Greeks values are `Decimal` (zero-floats invariant, issue #33).
//! Decimal inputs cross the prop boundary pre-formatted as `String`s.

use std::str::FromStr;

use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::primitive::badge::HfMoneynessBadgeProps;
use crate::primitive::{HfMoneynessBadge, Moneyness};
use crate::theme::tokens::HonbaStyle;

/// One side (CE or PE) of an option chain row. Decimal fields are
/// pre-formatted by the caller.
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct OptionLegQuote {
    pub ltp: String,        // Decimal, pre-formatted
    pub change_pct: String, // Decimal, pre-formatted
    pub volume: u64,
    pub open_interest: u64,
    pub oi_change: i64,
    pub iv: String,    // Decimal, pre-formatted
    pub delta: String, // Decimal, pre-formatted
    pub gamma: String, // Decimal, pre-formatted
}

/// The events of [`HfOptionChainRow`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfOptionChainRowEvent {
    /// An LTP cell was pressed: `(strike, is_call, is_buy)`. `strike` is the
    /// caller's pre-formatted decimal, passed back unchanged (`Decimal` has no
    /// `JsValue` impl, so it cannot be an event payload field).
    SelectLeg {
        strike: String,
        is_call: bool,
        is_buy: bool,
    },
}

/// Props for [`HfOptionChainRow`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfOptionChainRowProps {
    pub strike: Prop<String>,     // Decimal, pre-formatted
    pub spot_price: Prop<String>, // Decimal, pre-formatted
    pub call: Prop<OptionLegQuote>,
    pub put: Prop<OptionLegQuote>,
    /// Largest open interest on the chain, for scaling the OI bars. Defaults to
    /// the larger of this row's own call and put open interest.
    #[prop(default)]
    pub max_oi: Prop<Option<u64>>,
    #[prop(default = true)]
    pub is_buy: Prop<bool>,
}

/// Parse a pre-formatted decimal, `0` when unparsable.
fn parse(raw: &str) -> Decimal {
    Decimal::from_str(raw).unwrap_or_default()
}

/// A pre-formatted decimal rounded to `dp` places for display.
fn round_display(raw: &str, dp: u32) -> String {
    parse(raw).round_dp(dp).to_string()
}

/// Implied volatility as a percentage string, one decimal place.
fn iv_percent(raw: &str) -> String {
    format!("{}%", (parse(raw) * dec!(100)).round_dp(1))
}

/// Open-interest fill bar plus the signed OI change. The bar's width is a layout
/// percentage of `max_oi`, so it is plain `f64`.
fn oi_bar(quote: Prop<OptionLegQuote>, max_oi: impl Fn() -> u64 + 'static) -> impl IntoElement {
    let pct = computed(move || {
        let max = max_oi();
        if max > 0 {
            (quote.get().open_interest as f64 / max as f64 * 100.0).clamp(0.0, 100.0) as f32
        } else {
            0.0
        }
    });
    let fill = view()
        .flex_row()
        .w_full()
        .h(px(6.0))
        .rounded(px(3.0))
        .overflow_hidden()
        .surface_elevated()
        .child(
            view()
                .flex_grow(move || pct.get())
                .h_full()
                .bg(rgb(0x94a3b8)),
        )
        .child(view().flex_grow(move || 100.0 - pct.get()).h_full());

    view()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .text_size(px(10.0))
        .app_class(Some(SharedString::from("hf-oi-bar")))
        .child(fill.flex_1())
        .child(
            text(move || {
                let change = quote.get().oi_change;
                if change >= 0 {
                    format!("+{change}")
                } else {
                    change.to_string()
                }
            })
            .app_class(move || {
                let color = if quote.get().oi_change >= 0 {
                    "text-emerald-400"
                } else {
                    "text-rose-400"
                };
                Some(SharedString::from(color))
            }),
        )
}

/// One CE or PE straddle row: the call side (left) and put side (right) around a
/// center strike cell tagged with its moneyness.
#[component]
pub fn HfOptionChainRow(
    props: HfOptionChainRowProps,
    cx: Scope<HfOptionChainRowEvent>,
) -> impl IntoElement {
    let strike_raw = props.strike.get_untracked();
    let spot_raw = props.spot_price.get_untracked();
    let strike = parse(&strike_raw);
    let spot = parse(&spot_raw);

    let is_call_itm = spot > strike;
    let is_put_itm = spot < strike;
    let is_atm = (spot - strike).abs() < (strike * dec!(0.005));

    let call = props.call;
    let put = props.put;
    let is_buy = props.is_buy;

    // Same default as before: the larger of this row's own two open interests.
    let max_oi = move || {
        props
            .max_oi
            .get()
            .unwrap_or_else(|| call.get().open_interest.max(put.get().open_interest).max(1))
    };

    let moneyness = if is_atm {
        Moneyness::ATM
    } else if is_call_itm {
        Moneyness::ITM
    } else {
        Moneyness::OTM
    };

    let side_tint = |itm: bool, tint: &'static str| -> &'static str { if itm { tint } else { "" } };
    let call_tint = side_tint(is_call_itm, "bg-emerald-950/20");
    let put_tint = side_tint(is_put_itm, "bg-rose-950/20");

    let row_class = if is_atm {
        "hf-option-chain-row bg-surface-elevated font-semibold"
    } else {
        "hf-option-chain-row"
    };

    let cell = move |tint: &'static str| {
        view()
            .flex_1()
            .px(px(4.0))
            .py(px(4.0))
            .items_center()
            .app_class(Some(SharedString::from(format!("p-1 {tint}"))))
    };

    // Call LTP click zone: reports (strike, true, is_buy).
    let call_strike = strike_raw.clone();
    let call_ltp = view()
        .flex_1()
        .px(px(4.0))
        .py(px(4.0))
        .items_center()
        .app_class(Some(SharedString::from(
            "cursor-pointer hover:bg-emerald-500/10 p-1",
        )))
        .on_press(move |_: &()| {
            cx.emit(HfOptionChainRowEvent::SelectLeg {
                strike: call_strike.clone(),
                is_call: true,
                is_buy: is_buy.get_untracked(),
            });
        })
        .child(
            text(move || round_display(&call.get().ltp, 2))
                .font_family("monospace")
                .app_class(move || {
                    let color = if parse(&call.get().change_pct) >= Decimal::ZERO {
                        "text-emerald-400"
                    } else {
                        "text-rose-400"
                    };
                    Some(SharedString::from(color))
                }),
        );

    // Put LTP click zone: reports (strike, false, is_buy).
    let put_strike = strike_raw.clone();
    let put_ltp = view()
        .flex_1()
        .px(px(4.0))
        .py(px(4.0))
        .items_center()
        .app_class(Some(SharedString::from(
            "cursor-pointer hover:bg-rose-500/10 p-1",
        )))
        .on_press(move |_: &()| {
            cx.emit(HfOptionChainRowEvent::SelectLeg {
                strike: put_strike.clone(),
                is_call: false,
                is_buy: is_buy.get_untracked(),
            });
        })
        .child(
            text(move || round_display(&put.get().ltp, 2))
                .font_family("monospace")
                .app_class(move || {
                    let color = if parse(&put.get().change_pct) >= Decimal::ZERO {
                        "text-emerald-400"
                    } else {
                        "text-rose-400"
                    };
                    Some(SharedString::from(color))
                }),
        );

    let center = view()
        .flex_col()
        .items_center()
        .gap(px(2.0))
        .px(px(8.0))
        .py(px(4.0))
        .font_family("monospace")
        .font_weight(FontWeight::BOLD)
        .surface_muted()
        .border_soft()
        .app_class(Some(SharedString::from("hf-strike-cell")))
        .child(text(round_display(&strike_raw, 2)))
        .child(HfMoneynessBadge::element(HfMoneynessBadgeProps {
            moneyness: Prop::from(moneyness),
        }));

    view()
        .flex_row()
        .items_center()
        .app_class(Some(SharedString::from(row_class)))
        .child(cell(call_tint).child(text(move || round_display(&call.get().delta, 4))))
        .child(cell(call_tint).child(text(move || iv_percent(&call.get().iv))))
        .child(cell("").child(oi_bar(call, max_oi)))
        .child(call_ltp)
        .child(center)
        .child(put_ltp)
        .child(cell("").child(oi_bar(put, max_oi)))
        .child(cell(put_tint).child(text(move || iv_percent(&put.get().iv))))
        .child(cell(put_tint).child(text(move || round_display(&put.get().delta, 4))))
}
