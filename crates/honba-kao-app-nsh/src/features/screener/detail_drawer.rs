//! Screener detail drawer — right-side symbol panel (design doc §5.3).
//!
//! Plain reactive builder fn following the repo's `hf_tool_card` pattern:
//! no `#[derive(Props)]` / `#[component]` — the page mounts this inside a
//! `dynamic` region, so one-shot reads per mount are correct.
//!
//! The drawer resolves `active_symbol` against the screener `records` and
//! branches per asset class (stock / ETF / bond / mutual fund), sharing the
//! card shell, snapshot-grid cells, star toggle and quick-action buttons.

use std::rc::Rc;

use nsh_core::prelude::*;
use num_traits::ToPrimitive as _;

use honba_kao_ui_core::organisms::candlestick_chart::{
    ChartPeriod, HfCandlestickChart, HfCandlestickChartProps,
};
use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::primitive::sparkline::{HfSparklinePrimitive, HfSparklinePrimitiveProps};
use honba_kao_ui_core::theme::tokens::{Finish, Finished, HonbaStyle, MONO, colors, radius};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::features::fixtures;
use crate::features::screener::types::{
    ScreenerBondRecord, ScreenerEtfRecord, ScreenerInstrument, ScreenerMfRecord, ScreenerRecord,
    TechnicalRating, compact_cap_crore, compact_cr, compact_volume, inr, percent_text, signed_inr,
};

/// Chart-period chip labels, in `period_of` order.
const CHIPS: [(&str, ChartPeriod); 4] = [
    ("1D", ChartPeriod::Intraday),
    ("1W", ChartPeriod::M15),
    ("1M", ChartPeriod::D1),
    ("1Y", ChartPeriod::M1),
];

/// Chip index → chart period fed to the candlestick chart.
fn period_of(chip: u8) -> ChartPeriod {
    match chip {
        0 => ChartPeriod::Intraday,
        1 => ChartPeriod::M15,
        2 => ChartPeriod::D1,
        _ => ChartPeriod::M1,
    }
}

/// Consensus tone for a rating: sells tone down, neutral tones accent, buys tone up.
fn active_color(rating: TechnicalRating) -> AdaptiveColor {
    match rating {
        TechnicalRating::StrongSell | TechnicalRating::Sell => colors::DOWN,
        TechnicalRating::Neutral => colors::ACCENT,
        TechnicalRating::Buy | TechnicalRating::StrongBuy => colors::UP,
    }
}

/// Oscillator bucket counts (buy, neutral, sell) derived from the rating rank.
fn oscillator_counts(r: TechnicalRating) -> (u8, u8, u8) {
    match r {
        TechnicalRating::StrongBuy => (5, 1, 0),
        TechnicalRating::Buy => (4, 1, 1),
        TechnicalRating::Neutral => (2, 2, 2),
        TechnicalRating::Sell => (1, 1, 4),
        TechnicalRating::StrongSell => (0, 1, 5),
    }
}

/// Moving-average bucket counts (buy, neutral, sell) derived from the rating rank.
fn ma_counts(r: TechnicalRating) -> (u8, u8, u8) {
    match r {
        TechnicalRating::StrongBuy => (4, 0, 0),
        TechnicalRating::Buy => (3, 1, 0),
        TechnicalRating::Neutral => (2, 1, 1),
        TechnicalRating::Sell => (0, 1, 3),
        TechnicalRating::StrongSell => (0, 0, 4),
    }
}

/// Optional ratio → percent text, or an em dash placeholder.
fn opt_pct(value: Option<Decimal>) -> String {
    value
        .map(|v| format!("{}%", v.round_dp(2)))
        .unwrap_or_else(|| "—".into())
}

/// Signed percent with a direction arrow, for performance cells.
fn perf_pct(value: Decimal) -> String {
    format!(
        "{} {}",
        if value > Decimal::ZERO { "▲" } else { "▼" },
        percent_text(value)
    )
}

/// Up / down tone for a signed performance value.
fn perf_tone(value: Decimal) -> AdaptiveColor {
    if value > Decimal::ZERO {
        colors::UP
    } else {
        colors::DOWN
    }
}

/// Up / down tone for a signed change percent.
fn signed_tone(change_percent: Decimal) -> AdaptiveColor {
    if change_percent > Decimal::ZERO {
        colors::UP
    } else {
        colors::DOWN
    }
}

/// Shared SURFACE card shell — every section sets its own flex direction.
fn card_shell() -> View {
    view()
        .w_full()
        .bg(colors::SURFACE)
        .finish(Finish::Card)
        .rounded(px(radius::INSET))
        .border_1()
        .border_color(colors::BORDER)
        .p(px(8.0))
}

/// Snapshot-grid cell: muted label over a mono value (2×3 grids).
fn grid_cell(label: &'static str, value: String) -> View {
    grid_cell_colored(label, value, None)
}

/// Snapshot-grid cell with an explicit value tone (perf cells).
fn grid_cell_colored(label: &'static str, value: String, tone: Option<AdaptiveColor>) -> View {
    let value_el = match tone {
        Some(tone) => text(value)
            .text_size(px(12.0))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(tone)
            .font_family(MONO),
        None => text(value)
            .text_size(px(12.0))
            .font_weight(FontWeight::SEMIBOLD)
            .text_high()
            .font_family(MONO),
    };
    view()
        .w(px(160.0))
        .bg(colors::INSET)
        .rounded(px(radius::CONTROL))
        .p(px(6.0))
        .flex_col()
        .child(text(label).text_size(px(9.0)).text_muted())
        .child(value_el)
}

/// Wrapping row of snapshot-grid cells.
fn grid_row(cells: Vec<View>) -> View {
    view().flex_row().flex_wrap().gap(px(6.0)).children(cells)
}

/// Bucket tone segment row (equal `flex_1` widths, one segment per count).
fn bucket_segments(counts: (u8, u8, u8)) -> Vec<View> {
    let (buy, neutral, sell) = counts;
    let total = buy + neutral + sell;
    (0..total)
        .map(|at| {
            let tone = if at < buy {
                colors::UP
            } else if at < buy + neutral {
                colors::ACCENT
            } else {
                colors::DANGER
            };
            view().flex_1().h(px(6.0)).rounded(px(3.0)).bg(tone)
        })
        .collect()
}

/// Star toggle wired to the shortlist signal.
fn star_toggle(shortlisted: Signal<Vec<String>>, symbol: String, active: bool) -> View {
    view()
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(radius::CONTROL))
        .border_1()
        .border_color(colors::BORDER)
        .on_press(move || {
            shortlisted.update(|list| {
                let sym = symbol.clone();
                if let Some(at) = list.iter().position(|s| *s == sym) {
                    list.remove(at);
                } else {
                    list.push(sym);
                }
            });
        })
        .child(
            text(if active { "★" } else { "☆" })
                .text_size(px(12.0))
                .text_color(if active {
                    colors::ACCENT
                } else {
                    colors::MUTED
                }),
        )
}

/// Route jump button (cockpit / hub / …).
fn route_button(
    label: &'static str,
    route: &'static str,
    on_open: Rc<dyn Fn(&'static str)>,
) -> impl IntoElement {
    ghost_button(label, move || (on_open.clone())(route))
}

/// ＋ Watchlist toggle button.
fn watchlist_button(shortlisted: Signal<Vec<String>>, symbol: String) -> impl IntoElement {
    ghost_button("＋ Watchlist", move || {
        shortlisted.update(|list| {
            let sym = symbol.clone();
            if let Some(at) = list.iter().position(|s| *s == sym) {
                list.remove(at);
            } else {
                list.push(sym);
            }
        });
    })
}

/// Change pill: signed rupee value plus signed percent.
fn change_pill(change: Decimal, change_percent: Decimal, tone: AdaptiveColor) -> View {
    view()
        .px(px(6.0))
        .py(px(1.0))
        .rounded_full()
        .bg(colors::INSET)
        .child(
            text(format!(
                "{} ({})",
                signed_inr(change),
                percent_text(change_percent)
            ))
            .text_size(px(10.0))
            .text_color(tone),
        )
}

/// Period chip row (1D / 1W / 1M / 1Y).
fn period_chip_row(chip: Signal<u8>) -> View {
    view()
        .flex_row()
        .gap(px(4.0))
        .children((0u8..CHIPS.len() as u8).map(move |at| {
            view()
                .px(px(6.0))
                .py(px(1.0))
                .rounded(px(6.0))
                .bg(move || {
                    if chip.get() == at {
                        colors::ACCENT
                    } else {
                        colors::RAISED
                    }
                })
                .child(
                    text(CHIPS[at as usize].0)
                        .text_size(px(9.0))
                        .text_color(move || {
                            if chip.get() == at {
                                colors::ON_ACCENT
                            } else {
                                colors::MUTED
                            }
                        }),
                )
                .on_press(move || chip.set(at))
        }))
}

/// Candles / Area chart-kind chip row (stock drawer only).
fn chart_kind_row(chart_kind: Signal<u8>) -> View {
    view()
        .flex_row()
        .gap(px(4.0))
        .children((0u8..2).map(move |at| {
            let label = if at == 0 { "Candles" } else { "Area" };
            view()
                .px(px(6.0))
                .py(px(1.0))
                .rounded(px(6.0))
                .bg(move || {
                    if chart_kind.get() == at {
                        colors::ACCENT
                    } else {
                        colors::RAISED
                    }
                })
                .child(text(label).text_size(px(9.0)).text_color(move || {
                    if chart_kind.get() == at {
                        colors::ON_ACCENT
                    } else {
                        colors::MUTED
                    }
                }))
                .on_press(move || chart_kind.set(at))
        }))
}

/// A compact Ghost action button wired to one closure (the cockpit's
/// `HfButton::element(HfButtonProps { … })` call shape).
fn ghost_button(label: &'static str, action: impl Fn() + 'static) -> impl IntoElement {
    HfButton::element(HfButtonProps {
        href: Prop::default(),
        size: Prop::from(Some(ButtonSize::Sm)),
        variant: Prop::from(Some(ButtonVariant::Ghost)),
        full_width: Prop::from(false),
        disabled: Prop::from(false),
        aria_label: Prop::from(None),
        class: Prop::default(),
        children: Children::default(),
    })
    .child(text(label).text_size(px(11.0)))
    .on_event(move |event: &HfButtonEvent| {
        if *event == HfButtonEvent::Press {
            action();
        }
    })
}

#[allow(non_snake_case)]
pub fn ScreenerDetailDrawer(
    records: Vec<ScreenerRecord>,
    active_symbol: Signal<String>,
    shortlisted: Signal<Vec<String>>,
    drawer_open: Signal<bool>,
    on_open: Rc<dyn Fn(&'static str)>,
) -> View {
    let record = records
        .into_iter()
        .find(|r| r.symbol() == active_symbol.get());
    let Some(record) = record else {
        return view().flex_1().items_center().justify_center().child(
            text("Select an instrument to inspect")
                .text_muted()
                .text_size(px(11.0)),
        );
    };

    match record {
        ScreenerRecord::Stock(inst) => stock_drawer(inst, shortlisted, drawer_open, on_open),
        ScreenerRecord::Etf(etf) => etf_drawer(etf, shortlisted, drawer_open, on_open),
        ScreenerRecord::Bond(bond) => bond_drawer(bond, shortlisted, drawer_open, on_open),
        ScreenerRecord::MutualFund(mf) => mf_drawer(mf, shortlisted, drawer_open, on_open),
    }
}

// ── Stock ──────────────────────────────────────────────────────────────────
fn stock_drawer(
    inst: ScreenerInstrument,
    shortlisted: Signal<Vec<String>>,
    drawer_open: Signal<bool>,
    on_open: Rc<dyn Fn(&'static str)>,
) -> View {
    // Press closures are Fn, not FnOnce: keep copies of the mutating params.
    let shortlist_signal = shortlisted;
    let drawer_signal = drawer_open;

    // One-shot mount data (page remounts on instrument change).
    let star_symbol = inst.symbol.clone();
    let watch_symbol = inst.symbol.clone();
    let active = shortlist_signal.get().contains(&star_symbol);
    let change_tone = if inst.change_up() {
        colors::UP
    } else {
        colors::DOWN
    };

    let day_span = (inst.day_high - inst.day_low).max(dec!(0.01));
    let day_pos_dec = ((inst.price - inst.day_low) / day_span).clamp(dec!(0), dec!(1));
    let intraday_pos = day_pos_dec.to_f64().unwrap_or(0.0) as f32;
    let day_fill = (160.0 * intraday_pos).max(2.0);
    // 52-week markers for the 1Y chip (layout float at the canvas boundary).
    let fill_52 = (160.0 * inst.range_position()).max(2.0);
    let low_52 = inst.low_52;
    let high_52 = inst.high_52;

    let base = inst.price;
    let spark_closes = inst.sparkline_closes.clone();
    let change_up = inst.change_up();
    let chip = signal(0_u8);
    let chart_kind = signal(0_u8);

    // Consensus inputs: borrows first, then the (possibly moving) rating value.
    let (buys, sells) = inst.technical_rating.counts();
    let rank = inst.technical_rating.rank();
    let rating_text = inst.technical_rating.label();
    let consensus_tone = active_color(inst.technical_rating);
    let osc = oscillator_counts(inst.technical_rating);
    let ma = ma_counts(inst.technical_rating);

    // ── Section 1 · header card ─────────────────────────────────────────────
    let header_card = card_shell()
        .flex_row()
        .justify_between()
        .child(
            view().flex_col().gap(px(3.0)).child(
                view()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        text(inst.symbol.clone())
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_high(),
                    )
                    .child(
                        view()
                            .rounded(px(6.0))
                            .bg(colors::INSET)
                            .px(px(4.0))
                            .py(px(1.0))
                            .child(text(inst.exchange.clone()).text_size(px(9.0)).text_muted()),
                    ),
            ),
        )
        .child(text(inst.name.clone()).text_size(px(11.0)).text_muted())
        .child(
            view()
                .rounded(px(6.0))
                .bg(colors::INSET)
                .px(px(6.0))
                .py(px(1.0))
                .child(
                    text(format!("Sector: {}", inst.sector))
                        .text_size(px(9.0))
                        .text_muted(),
                ),
        )
        // Header right: star toggle + close.
        .child(star_toggle(shortlist_signal, star_symbol, active))
        .child(ghost_button("✕", move || {
            drawer_signal.set(false);
        }));

    // ── Section 2 · live quote box ─────────────────────────────────────────
    let quote_card = card_shell()
        .flex_col()
        .gap(px(6.0))
        .child(
            view().flex_row().items_center().gap(px(6.0)).child(
                text(inr(inst.price))
                    .text_size(px(22.0))
                    .font_weight(FontWeight::BOLD)
                    .text_high()
                    .font_family(MONO),
            ),
        )
        .child(change_pill(inst.change, inst.change_percent, change_tone))
        // Range bar: day's range on 1D, 52-week range on 1Y, hidden on 1W/1M.
        .child(dynamic(move || {
            if chip.get() == 1 || chip.get() == 2 {
                return view();
            }
            let is_52w = chip.get() == 3;
            let caption = if is_52w {
                "52-Week Range"
            } else {
                "Day's Range"
            };
            let low_label = if is_52w { inr(low_52) } else { "L".to_string() };
            let high_label = if is_52w {
                inr(high_52)
            } else {
                "H".to_string()
            };
            let fill = if is_52w { fill_52 } else { day_fill };
            view()
                .flex_col()
                .gap(px(3.0))
                .child(text(caption).text_size(px(9.0)).text_muted())
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .child(text(low_label).text_muted().text_size(px(9.0)))
                        .child(
                            view()
                                .w(px(160.0))
                                .h(px(4.0))
                                .rounded_full()
                                .bg(colors::RAISED)
                                .overflow_hidden()
                                .child(view().h_full().w(px(fill)).bg(colors::ACCENT)),
                        )
                        .child(text(high_label).text_muted().text_size(px(9.0))),
                )
        }));

    // ── Section 3 · mini chart card ────────────────────────────────────────
    let chart_card = card_shell()
        .flex_col()
        .gap(px(6.0))
        .child(
            view()
                .flex_row()
                .gap(px(6.0))
                .child(chart_kind_row(chart_kind))
                .child(period_chip_row(chip)),
        )
        .child(dynamic(move || {
            if chart_kind.get() == 0 {
                view()
                    .w_full()
                    .child(HfCandlestickChart::element(HfCandlestickChartProps {
                        candles: Prop::bind(move || {
                            fixtures::candles_for_period(base, period_of(chip.get()))
                        }),
                        period: Prop::bind(move || period_of(chip.get())),
                        show_volume: Prop::from(true),
                        show_averages: Prop::from(true),
                        prev_close: Prop::from(String::new()),
                        height: 240.0,
                    }))
            } else {
                view()
                    .w_full()
                    .h(px(236.0))
                    .child(HfSparklinePrimitive::element(HfSparklinePrimitiveProps {
                        values: Prop::from(spark_closes.clone()),
                        height: Prop::from(Some(236.0_f32)),
                        color: Prop::from(if change_up {
                            None::<SharedString>
                        } else {
                            Some("text-rose-500".into())
                        }),
                    }))
            }
        }));

    // ── Section 4 · consensus meter ────────────────────────────────────────
    let consensus_card = card_shell()
        .flex_col()
        .gap(px(4.0))
        .child(
            text("Technical Consensus")
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_muted(),
        )
        .child(
            view()
                .flex_row()
                .gap(px(2.0))
                .children((0u8..5).map(move |segment| {
                    view()
                        .flex_1()
                        .h(px(6.0))
                        .rounded(px(3.0))
                        .bg(if segment == rank {
                            consensus_tone
                        } else {
                            colors::RAISED
                        })
                })),
        )
        .child(
            text(rating_text.to_string())
                .text_size(px(12.0))
                .font_weight(FontWeight::BOLD)
                .text_color(consensus_tone),
        )
        .child(
            text(format!(
                "Bullish: {} Buy indicators, {} Sell indicators",
                buys, sells
            ))
            .text_size(px(10.0))
            .text_muted(),
        );

    // ── Section 4b · oscillator & moving-average breakdown ─────────────────
    let indicators_card = view()
        .flex_row()
        .gap(px(6.0))
        .child(
            view()
                .w(px(160.0))
                .bg(colors::INSET)
                .rounded(px(radius::CONTROL))
                .p(px(6.0))
                .flex_col()
                .gap(px(4.0))
                .child(text("Oscillators").text_size(px(9.0)).text_muted())
                .child(
                    text(format!(
                        "{} Buy · {} Neutral · {} Sell",
                        osc.0, osc.1, osc.2
                    ))
                    .text_size(px(10.0))
                    .text_high(),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(2.0))
                        .children(bucket_segments(osc)),
                ),
        )
        .child(
            view()
                .w(px(160.0))
                .bg(colors::INSET)
                .rounded(px(radius::CONTROL))
                .p(px(6.0))
                .flex_col()
                .gap(px(4.0))
                .child(text("Moving Averages").text_size(px(9.0)).text_muted())
                .child(
                    text(format!("{} Buy · {} Neutral · {} Sell", ma.0, ma.1, ma.2))
                        .text_size(px(10.0))
                        .text_high(),
                )
                .child(view().flex_row().gap(px(2.0)).children(bucket_segments(ma))),
        );

    // ── Section 5 · key ratios 2×3 grid ────────────────────────────────────
    let ratios_card = card_shell().flex_col().gap(px(6.0)).child(
        view()
            .flex_row()
            .flex_wrap()
            .gap(px(6.0))
            .child(
                view()
                    .w(px(160.0))
                    .bg(colors::INSET)
                    .rounded(px(radius::CONTROL))
                    .p(px(6.0))
                    .flex_col()
                    .child(text("Market Cap").text_size(px(9.0)).text_muted())
                    .child(
                        text(compact_cap_crore(inst.market_cap_cr))
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_high()
                            .font_family(MONO),
                    ),
            )
            .child(
                view()
                    .w(px(160.0))
                    .bg(colors::INSET)
                    .rounded(px(radius::CONTROL))
                    .p(px(6.0))
                    .flex_col()
                    .child(text("P/E").text_size(px(9.0)).text_muted())
                    .child(
                        text(
                            inst.pe_ratio
                                .map(|v| v.round_dp(2).to_string())
                                .unwrap_or_else(|| "—".into()),
                        )
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_high()
                        .font_family(MONO),
                    ),
            )
            .child(
                view()
                    .w(px(160.0))
                    .bg(colors::INSET)
                    .rounded(px(radius::CONTROL))
                    .p(px(6.0))
                    .flex_col()
                    .child(text("P/B").text_size(px(9.0)).text_muted())
                    .child(
                        text(
                            inst.pb_ratio
                                .map(|v| v.round_dp(2).to_string())
                                .unwrap_or_else(|| "—".into()),
                        )
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_high()
                        .font_family(MONO),
                    ),
            )
            .child(
                view()
                    .w(px(160.0))
                    .bg(colors::INSET)
                    .rounded(px(radius::CONTROL))
                    .p(px(6.0))
                    .flex_col()
                    .child(text("ROE").text_size(px(9.0)).text_muted())
                    .child(
                        text(opt_pct(inst.roe))
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_high()
                            .font_family(MONO),
                    ),
            )
            .child(
                view()
                    .w(px(160.0))
                    .bg(colors::INSET)
                    .rounded(px(radius::CONTROL))
                    .p(px(6.0))
                    .flex_col()
                    .child(text("ROCE").text_size(px(9.0)).text_muted())
                    .child(
                        text(opt_pct(inst.roce))
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_high()
                            .font_family(MONO),
                    ),
            )
            .child(
                view()
                    .w(px(160.0))
                    .bg(colors::INSET)
                    .rounded(px(radius::CONTROL))
                    .p(px(6.0))
                    .flex_col()
                    .child(text("52W Range").text_size(px(9.0)).text_muted())
                    .child(
                        text(format!("{} – {}", inr(inst.low_52), inr(inst.high_52)))
                            .text_size(px(9.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_high()
                            .font_family(MONO),
                    ),
            ),
    );

    // ── Section 6 · workstation quick actions ──────────────────────────────
    let actions_card = card_shell()
        .flex_row()
        .gap(px(6.0))
        .child(route_button(
            "Open Cockpit",
            crate::router::paths::ROUTE_TRADING_COCKPIT,
            Rc::clone(&on_open),
        ))
        .child(route_button(
            "Security Hub",
            crate::router::paths::ROUTE_RESEARCH_SECURITY,
            Rc::clone(&on_open),
        ))
        .child(route_button(
            "Option Chain",
            crate::router::paths::ROUTE_DERIVATIVES_OPTION_CHAIN,
            Rc::clone(&on_open),
        ))
        .child(watchlist_button(shortlist_signal, watch_symbol));

    // ── Root: vertical scroll of cards ─────────────────────────────────────
    view()
        .flex_col()
        .h_full()
        .w_full()
        .bg(colors::BACKGROUND)
        .child(
            scroll()
                .flex_1()
                .flex_col()
                .gap(px(6.0))
                .p(px(8.0))
                .child(header_card)
                .child(quote_card)
                .child(chart_card)
                .child(consensus_card)
                .child(indicators_card)
                .child(ratios_card)
                .child(actions_card),
        )
}

// ── ETF ────────────────────────────────────────────────────────────────────
fn etf_drawer(
    etf: ScreenerEtfRecord,
    shortlisted: Signal<Vec<String>>,
    drawer_open: Signal<bool>,
    on_open: Rc<dyn Fn(&'static str)>,
) -> View {
    let shortlist_signal = shortlisted;
    let star_symbol = etf.symbol.clone();
    let watch_symbol = etf.symbol.clone();
    let active = shortlist_signal.get().contains(&star_symbol);
    let change_tone = signed_tone(etf.change_percent);
    let base = etf.nav;
    let chip = signal(0_u8);

    // ── Section 1 · header card ─────────────────────────────────────────────
    let header_card = card_shell()
        .flex_row()
        .justify_between()
        .child(
            view().flex_col().gap(px(3.0)).child(
                view()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        text(etf.symbol.clone())
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_high(),
                    )
                    .child(
                        view()
                            .rounded(px(6.0))
                            .bg(colors::INSET)
                            .px(px(4.0))
                            .py(px(1.0))
                            .child(text("ETF").text_size(px(9.0)).text_muted()),
                    ),
            ),
        )
        .child(text(etf.name.clone()).text_size(px(11.0)).text_muted())
        .child(
            view()
                .rounded(px(6.0))
                .bg(colors::INSET)
                .px(px(6.0))
                .py(px(1.0))
                .child(
                    text(format!("{} · {}", etf.brand, etf.focus))
                        .text_size(px(9.0))
                        .text_muted(),
                ),
        )
        .child(star_toggle(shortlist_signal, star_symbol, active))
        .child(ghost_button("✕", move || {
            drawer_open.set(false);
        }));

    // ── Section 2 · quote box (NAV, no range bar) ──────────────────────────
    let quote_card = card_shell()
        .flex_col()
        .gap(px(6.0))
        .child(
            view().flex_row().items_center().gap(px(6.0)).child(
                text(inr(etf.nav))
                    .text_size(px(22.0))
                    .font_weight(FontWeight::BOLD)
                    .text_high()
                    .font_family(MONO),
            ),
        )
        .child(change_pill(etf.change, etf.change_percent, change_tone));

    // ── Section 3 · mini chart card ────────────────────────────────────────
    let chart_card = card_shell()
        .flex_col()
        .gap(px(6.0))
        .child(period_chip_row(chip))
        .child(HfCandlestickChart::element(HfCandlestickChartProps {
            candles: Prop::bind(move || fixtures::candles_for_period(base, period_of(chip.get()))),
            period: Prop::bind(move || period_of(chip.get())),
            show_volume: Prop::from(true),
            show_averages: Prop::from(true),
            prev_close: Prop::from(String::new()),
            height: 240.0,
        }));

    // ── Section 4 · ETF snapshot 2×3 grid ──────────────────────────────────
    let snapshot_card = card_shell()
        .flex_col()
        .gap(px(6.0))
        .child(
            text("ETF Snapshot")
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_muted(),
        )
        .child(grid_row(vec![
            grid_cell("AUM", compact_cr(etf.aum_cr)),
            grid_cell(
                "Expense Ratio",
                format!("{}%", etf.expense_ratio.round_dp(2)),
            ),
            grid_cell(
                "Tracking Error",
                format!("{}%", etf.tracking_error.round_dp(2)),
            ),
            grid_cell_colored(
                "1Y Perf",
                perf_pct(etf.perf_1y),
                Some(perf_tone(etf.perf_1y)),
            ),
            grid_cell_colored(
                "3Y Perf",
                perf_pct(etf.perf_3y),
                Some(perf_tone(etf.perf_3y)),
            ),
            grid_cell("Volume", compact_volume(etf.volume)),
        ]));

    // ── Section 5 · workstation quick actions ──────────────────────────────
    let actions_card = card_shell()
        .flex_row()
        .gap(px(6.0))
        .child(route_button(
            "Open Cockpit",
            crate::router::paths::ROUTE_TRADING_COCKPIT,
            Rc::clone(&on_open),
        ))
        .child(route_button(
            "Security Hub",
            crate::router::paths::ROUTE_RESEARCH_SECURITY,
            Rc::clone(&on_open),
        ))
        .child(watchlist_button(shortlist_signal, watch_symbol));

    view()
        .flex_col()
        .h_full()
        .w_full()
        .bg(colors::BACKGROUND)
        .child(
            scroll()
                .flex_1()
                .flex_col()
                .gap(px(6.0))
                .p(px(8.0))
                .child(header_card)
                .child(quote_card)
                .child(chart_card)
                .child(snapshot_card)
                .child(actions_card),
        )
}

// ── Bond ───────────────────────────────────────────────────────────────────
fn bond_drawer(
    bond: ScreenerBondRecord,
    shortlisted: Signal<Vec<String>>,
    drawer_open: Signal<bool>,
    on_open: Rc<dyn Fn(&'static str)>,
) -> View {
    let shortlist_signal = shortlisted;
    let star_symbol = bond.symbol.clone();
    let watch_symbol = bond.symbol.clone();
    let active = shortlist_signal.get().contains(&star_symbol);
    let change_tone = signed_tone(bond.change_percent);

    // ── Section 1 · header card (issuer + rating pills) ────────────────────
    let header_card = card_shell()
        .flex_row()
        .justify_between()
        .child(
            view().flex_col().gap(px(3.0)).child(
                view().flex_row().items_center().gap(px(4.0)).child(
                    text(bond.symbol.clone())
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_high(),
                ),
            ),
        )
        .child(text(bond.name.clone()).text_size(px(11.0)).text_muted())
        .child(
            view()
                .flex_row()
                .gap(px(4.0))
                .child(
                    view()
                        .rounded(px(6.0))
                        .bg(colors::INSET)
                        .px(px(6.0))
                        .py(px(1.0))
                        .child(
                            text(bond.issuer_type.label().to_string())
                                .text_size(px(9.0))
                                .text_muted(),
                        ),
                )
                .child(
                    view()
                        .rounded(px(6.0))
                        .bg(colors::INSET)
                        .px(px(6.0))
                        .py(px(1.0))
                        .child(
                            text(bond.credit_rating.label().to_string())
                                .text_size(px(9.0))
                                .text_muted(),
                        ),
                ),
        )
        .child(star_toggle(shortlist_signal, star_symbol, active))
        .child(ghost_button("✕", move || {
            drawer_open.set(false);
        }));

    // ── Section 2 · quote box (clean price, no chart) ──────────────────────
    let quote_card = card_shell()
        .flex_col()
        .gap(px(6.0))
        .child(
            view().flex_row().items_center().gap(px(6.0)).child(
                text(inr(bond.clean_price))
                    .text_size(px(22.0))
                    .font_weight(FontWeight::BOLD)
                    .text_high()
                    .font_family(MONO),
            ),
        )
        .child(
            view()
                .px(px(6.0))
                .py(px(1.0))
                .rounded_full()
                .bg(colors::INSET)
                .child(
                    text(percent_text(bond.change_percent))
                        .text_size(px(10.0))
                        .text_color(change_tone),
                ),
        );

    // ── Section 3 · bond term sheet 2×3 grid ───────────────────────────────
    let term_card = card_shell()
        .flex_col()
        .gap(px(6.0))
        .child(
            text("Bond Term Sheet")
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_muted(),
        )
        .child(grid_row(vec![
            grid_cell("Coupon", format!("{}%", bond.coupon_rate.round_dp(2))),
            grid_cell("YTW", format!("{}%", bond.ytw.round_dp(2))),
            grid_cell("Clean Price", inr(bond.clean_price)),
            grid_cell("Face Value", inr(bond.face_value)),
            grid_cell("Maturity", bond.maturity_date.clone()),
            grid_cell("Duration", format!("{}y", bond.duration_years.round_dp(1))),
        ]));

    // ── Section 4 · workstation quick actions ──────────────────────────────
    let actions_card = card_shell()
        .flex_row()
        .gap(px(6.0))
        .child(route_button(
            "Open Cockpit",
            crate::router::paths::ROUTE_TRADING_COCKPIT,
            Rc::clone(&on_open),
        ))
        .child(watchlist_button(shortlist_signal, watch_symbol));

    view()
        .flex_col()
        .h_full()
        .w_full()
        .bg(colors::BACKGROUND)
        .child(
            scroll()
                .flex_1()
                .flex_col()
                .gap(px(6.0))
                .p(px(8.0))
                .child(header_card)
                .child(quote_card)
                .child(term_card)
                .child(actions_card),
        )
}

// ── Mutual fund ────────────────────────────────────────────────────────────
fn mf_drawer(
    mf: ScreenerMfRecord,
    shortlisted: Signal<Vec<String>>,
    drawer_open: Signal<bool>,
    on_open: Rc<dyn Fn(&'static str)>,
) -> View {
    let shortlist_signal = shortlisted;
    let star_symbol = mf.symbol.clone();
    let watch_symbol = mf.symbol.clone();
    let active = shortlist_signal.get().contains(&star_symbol);
    let change_tone = signed_tone(mf.change_percent);

    // Star strip for the header sub-line (guarded to 1..=5).
    let star_line = {
        let n = mf.rating_stars;
        if (1..=5).contains(&n) {
            format!("{}{}", "★".repeat(n as usize), "☆".repeat(5 - n as usize))
        } else {
            String::new()
        }
    };
    let sub_line = if star_line.is_empty() {
        format!("{} · {}", mf.amc, mf.category)
    } else {
        format!("{} · {} · {}", star_line, mf.amc, mf.category)
    };

    // ── Section 1 · header card (scheme + star pills) ──────────────────────
    let header_card = card_shell()
        .flex_row()
        .justify_between()
        .child(
            view().flex_col().gap(px(3.0)).child(
                view()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        text(mf.symbol.clone())
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_high(),
                    )
                    .child(
                        view()
                            .rounded(px(6.0))
                            .bg(colors::INSET)
                            .px(px(4.0))
                            .py(px(1.0))
                            .child(
                                text(mf.scheme_type.label().to_string())
                                    .text_size(px(9.0))
                                    .text_muted(),
                            ),
                    )
                    .child(
                        view()
                            .rounded(px(6.0))
                            .bg(colors::INSET)
                            .px(px(4.0))
                            .py(px(1.0))
                            .child(
                                text(format!("★ × {}", mf.rating_stars))
                                    .text_size(px(9.0))
                                    .text_muted(),
                            ),
                    ),
            ),
        )
        .child(text(mf.name.clone()).text_size(px(11.0)).text_muted())
        .child(
            view()
                .rounded(px(6.0))
                .bg(colors::INSET)
                .px(px(6.0))
                .py(px(1.0))
                .child(text(sub_line).text_size(px(9.0)).text_muted()),
        )
        .child(star_toggle(shortlist_signal, star_symbol, active))
        .child(ghost_button("✕", move || {
            drawer_open.set(false);
        }));

    // ── Section 2 · quote box (NAV) ────────────────────────────────────────
    let quote_card = card_shell()
        .flex_col()
        .gap(px(6.0))
        .child(
            view().flex_row().items_center().gap(px(6.0)).child(
                text(inr(mf.nav))
                    .text_size(px(22.0))
                    .font_weight(FontWeight::BOLD)
                    .text_high()
                    .font_family(MONO),
            ),
        )
        .child(change_pill(mf.change, mf.change_percent, change_tone));

    // ── Section 3 · chart placeholder ──────────────────────────────────────
    let chart_placeholder = card_shell()
        .flex_col()
        .child(text("(no chart)").text_size(px(10.0)).text_muted());

    // ── Section 4 · scheme snapshot 2×3 grid ───────────────────────────────
    let snapshot_card = card_shell()
        .flex_col()
        .gap(px(6.0))
        .child(
            text("Scheme Snapshot")
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_muted(),
        )
        .child(grid_row(vec![
            grid_cell("AUM", compact_cr(mf.aum_cr)),
            grid_cell("Expense", format!("{}%", mf.expense_ratio.round_dp(2))),
            grid_cell("Sharpe", mf.sharpe_ratio.round_dp(2).to_string()),
            grid_cell_colored("CAGR 1Y", perf_pct(mf.cagr_1y), Some(perf_tone(mf.cagr_1y))),
            grid_cell_colored("CAGR 3Y", perf_pct(mf.cagr_3y), Some(perf_tone(mf.cagr_3y))),
            grid_cell_colored("CAGR 5Y", perf_pct(mf.cagr_5y), Some(perf_tone(mf.cagr_5y))),
        ]));

    // ── Section 5 · workstation quick actions ──────────────────────────────
    let actions_card = card_shell()
        .flex_row()
        .gap(px(6.0))
        .child(route_button(
            "Open Mutual Funds",
            crate::router::paths::ROUTE_WEALTH_MUTUAL_FUNDS,
            Rc::clone(&on_open),
        ))
        .child(watchlist_button(shortlist_signal, watch_symbol));

    view()
        .flex_col()
        .h_full()
        .w_full()
        .bg(colors::BACKGROUND)
        .child(
            scroll()
                .flex_1()
                .flex_col()
                .gap(px(6.0))
                .p(px(8.0))
                .child(header_card)
                .child(quote_card)
                .child(chart_placeholder)
                .child(snapshot_card)
                .child(actions_card),
        )
}
