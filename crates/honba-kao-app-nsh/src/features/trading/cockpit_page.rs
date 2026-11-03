//! Trading Cockpit — the flagship three-column trading workstation
//! (Stage 3 Phase A.3 of the gallery-rs porting guide, replacing the
//! listing's `HfThreeColumnTradingLayout` skeleton).
//!
//! Left (`HfThreeColumnChartSlot`): the pool's native-canvas
//! [`HfCandlestickChart`] (M5 candles, volume + MA overlays) on
//! deterministic Decimal fixtures. Center (`HfThreeColumnDepthSlot`):
//! the [`HfOrderBook`] L2 ladder plus the [`HfMarketTape`] print
//! stream. Right (`HfThreeColumnTicketSlot`): the [`HfOrderTicket`]
//! with a Decimal-derived estimation strip — quantity × LTP and the
//! marginal charge, Decimal arithmetic only, per the zero-floats
//! invariant.
//! Below: the [`HfTradingDock`] whose panes read the context's live
//! positions/orders signals (empty-safe).
//!
//! The timeframe stepper cycles [`ChartPeriod`] in the chart slot
//! (Intraday/M1/M5/M15/D1; M15 and D1 re-sample the deterministic walk
//! through `candles_for_period`). The L2 ladder binds to the depth
//! socket over the Phase A.1 seam — `HonbaClientContext::depth` written
//! by the web transport's `DepthFeed` websocket (`/ws/live`,
//! `NSE:RELIANCE-EQ`) — falling back to the engine ladder while the
//! socket is cold; the tape still renders the shared Decimal fixtures
//! (`crate::features::fixtures`).

use crate::features::fixtures;
use crate::shared::contexts::HonbaClientContext;
use honba_kao_client::dto::market::{BookLevelDto, Level2Book};
use honba_kao_client::dto::order::OrderIntent;
use honba_kao_ui_core::organisms::candlestick_chart::{
    ChartPeriod, HfCandlestickChart, HfCandlestickChartProps,
};
use honba_kao_ui_core::organisms::market_tape::{HfMarketTape, HfMarketTapeProps};
use honba_kao_ui_core::organisms::order_book::{
    BookLevel, HfOrderBook, HfOrderBookProps, SpreadInfo,
};
use honba_kao_ui_core::organisms::order_ticket::{HfOrderTicket, HfOrderTicketProps};
use honba_kao_ui_core::organisms::trading_dock::{HfTradingDock, HfTradingDockProps};
use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::templates::three_columns::{
    HfThreeColumnChartSlot, HfThreeColumnChartSlotProps, HfThreeColumnDepthSlot,
    HfThreeColumnDepthSlotProps, HfThreeColumnTicketSlot, HfThreeColumnTicketSlotProps,
    HfThreeColumnTradingLayout, HfThreeColumnTradingLayoutProps,
};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Facade: fixtures is `pub(crate)`, so integration tests reach the period
// re-sampler through this module (`features::trading::cockpit_page::…`).
pub use crate::features::fixtures::candles_for_period;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct CockpitPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Trading Cockpit — institutional order entry, L2 depth, chart
/// visualizer, market tape, and positions dock.
#[component]
pub fn CockpitPage(_props: CockpitPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let client = use_context::<HonbaClientContext>().expect("client context");

    let base = dec!(22842.50);
    let tape_entries = fixtures::tape();

    // The ticket's Decimal estimate: quantity x LTP (zero floats).
    let est_value = fixtures::fmt(Decimal::from(100) * base);
    // The submit/depth closures outlive the initial build; give each its
    // own context copy so the dock's row closures keep theirs.
    let client_submit = client.clone();
    let client_orders = client.clone();
    let client_positions = client.clone();
    let client_depth = client;

    // The timeframe stepper: ◀ steps back, the chips select, ▶ steps on —
    // all five periods cycle through PERIOD_CYCLE.
    let period = signal(ChartPeriod::M5);
    let step_in_cycle = |dir: i32| {
        move |event: &HfButtonEvent| {
            if *event == HfButtonEvent::Press {
                let at = PERIOD_CYCLE
                    .iter()
                    .position(|candidate| *candidate == period.get())
                    .unwrap_or_default();
                let shifted = (at as i32 + dir).rem_euclid(PERIOD_CYCLE.len() as i32) as usize;
                period.set(PERIOD_CYCLE[shifted]);
            }
        }
    };
    let step_button = |label: &'static str, dir: i32| {
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
        .child(text(label))
        .on_event(step_in_cycle(dir))
    };
    let mut period_toolbar = view().flex_row().gap_2().child(step_button("◀", -1));
    for candidate in PERIOD_CYCLE {
        let chip_variant = period;
        period_toolbar = period_toolbar.child(
            HfButton::element(HfButtonProps {
                href: Prop::default(),
                size: Prop::from(Some(ButtonSize::Sm)),
                variant: Prop::bind(move || {
                    if chip_variant.get() == candidate {
                        Some(ButtonVariant::Primary)
                    } else {
                        Some(ButtonVariant::Ghost)
                    }
                }),
                full_width: Prop::from(false),
                disabled: Prop::from(false),
                aria_label: Prop::from(None),
                class: Prop::default(),
                children: Children::default(),
            })
            .child(text(period_label(candidate)))
            .on_event(move |event: &HfButtonEvent| {
                if *event == HfButtonEvent::Press {
                    period.set(candidate);
                }
            }),
        );
    }
    let period_toolbar = period_toolbar.child(step_button("▶", 1));

    let chart = HfCandlestickChart::element(HfCandlestickChartProps {
        candles: Prop::bind(move || fixtures::candles_for_period(base, period.get())),
        period: Prop::bind(move || period.get()),
        show_volume: Prop::from(true),
        show_averages: Prop::from(true),
        prev_close: Prop::from(fixtures::fmt(dec!(22800.00))),
        height: 260.0,
    });

    // The L2 ladder reads the context's depth socket; while the socket is
    // cold (offline runs, cold web transport) it falls back to the engine
    // ladder, and the caption names the live source.
    let depth_live = move || match client_depth.depth.get() {
        Some(book) if !book.bids.is_empty() && !book.asks.is_empty() => book_view(&book),
        _ => fixtures::book(base),
    };
    let caption_client = client_depth.clone();

    let depth = view()
        .flex_col()
        .gap_2()
        .child(HfOrderBook::element(HfOrderBookProps {
            bids: Prop::bind(move || depth_live().0),
            asks: Prop::bind(move || depth_live().1),
            spread: Prop::bind(move || depth_live().2),
        }))
        .child(HfMarketTape::element(HfMarketTapeProps {
            entries: Prop::from(tape_entries),
        }))
        .child(
            text(move || {
                if matches!(
                    caption_client.depth.get(),
                    Some(book) if !book.bids.is_empty() && !book.asks.is_empty()
                ) {
                    "Depth: live L2 socket".to_string()
                } else {
                    "Depth: fixture ladder (no socket)".to_string()
                }
            })
            .text_size(px(12.0))
            .text_muted(),
        );

    let ticket = HfOrderTicket::element(HfOrderTicketProps {
        open: Prop::from(None::<bool>),
        default_open: Prop::from(true),
        symbol: Prop::from("RELIANCE".to_string()),
        side: Prop::from("BUY".to_string()),
        quantity: Prop::from(fixtures::fmt(Decimal::from(100))),
        children: Children::default(),
    })
    .child(
        view()
            .flex_col()
            .gap_2()
            .p_3()
            .child(
                text("Est. value ₹100 × LTP")
                    .text_size(px(12.0))
                    .text_muted(),
            )
            .child(
                text(format!("₹{est_value}"))
                    .font_family("monospace")
                    .text_size(px(15.0))
                    .text_high(),
            )
            .child(
                HfButton::element(HfButtonProps {
                    href: Prop::default(),
                    size: Prop::from(Some(ButtonSize::Md)),
                    variant: Prop::from(Some(ButtonVariant::Primary)),
                    full_width: Prop::from(false),
                    disabled: Prop::from(false),
                    aria_label: Prop::from(None),
                    class: Prop::default(),
                    children: Children::default(),
                })
                .child(text("Place Order"))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        // The deterministic demo slice: a LIMIT BUY 100
                        // RELIANCE @ the chart's base. The context's
                        // submit goes over the injected transport (the
                        // offline mirror warn-logs and drops it).
                        (client_submit.submit_order)(OrderIntent {
                            symbol: "RELIANCE".to_string(),
                            side: "BUY".to_string(),
                            order_type: "LIMIT".to_string(),
                            price: Some(dec!(22842.50)),
                            trigger_price: None,
                            quantity: Decimal::from(100),
                        });
                    }
                }),
            ),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Trading Cockpit".to_string())),
        subtitle: Prop::from(Some(
            "Order entry, L2 depth ladder, chart visualizer, market tape, and positions dock"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfThreeColumnTradingLayout::element(HfThreeColumnTradingLayoutProps {
            market_tape: Prop::from(Some(fixtures::tape_strip())),
            children: Children::default(),
        })
        .child(
            HfThreeColumnChartSlot::element(HfThreeColumnChartSlotProps {
                children: Children::default(),
            })
            .child(period_toolbar)
            .child(chart),
        )
        .child(
            HfThreeColumnDepthSlot::element(HfThreeColumnDepthSlotProps {
                children: Children::default(),
            })
            .child(depth),
        )
        .child(
            HfThreeColumnTicketSlot::element(HfThreeColumnTicketSlotProps {
                children: Children::default(),
            })
            .child(ticket),
        ),
    )
    .child(
        HfTradingDock::element(HfTradingDockProps {
            children: Children::default(),
        })
        .child(
            view()
                .flex_col()
                .gap_2()
                .child(
                    text("Positions")
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_muted(),
                )
                .child(dock_list(move || {
                    client_positions
                        .positions
                        .get()
                        .iter()
                        .map(|p| {
                            format!(
                                "{}  ·  {}  ·  qty {} @ {} (mkt {}) [{}]",
                                p.symbol,
                                p.side,
                                fixtures::fmt(p.quantity),
                                fixtures::fmt(p.entry_price),
                                fixtures::fmt(p.current_price),
                                fixtures::fmt(p.quantity * p.entry_price),
                            )
                        })
                        .collect()
                }))
                .child(
                    text("Working Orders")
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_muted(),
                )
                .child(dock_list(move || {
                    client_orders
                        .orders
                        .get()
                        .iter()
                        .map(|o| {
                            let status = format!("{:?}", o.status);
                            let price = o
                                .price
                                .map(fixtures::fmt)
                                .unwrap_or_else(|| "MKT".to_string());
                            format!(
                                "{}  ·  {}  ·  {}  ·  {}  ·  {}",
                                o.order_id, o.symbol, o.side, status, price
                            )
                        })
                        .collect()
                })),
        ),
    )
}

/// The dock's shared empty-state-aware row list rendering.
fn dock_list(rows: impl Fn() -> Vec<String> + 'static) -> impl IntoElement {
    text(move || {
        let rows = rows();
        if rows.is_empty() {
            "Feed idle — no rows yet".to_string()
        } else {
            rows.join("\n")
        }
    })
    .font_family("monospace")
    .text_size(px(12.0))
    .text_muted()
}

/// The timeframe stepper's canonical cycle: Intraday → M1 → M5 → M15 → D1
/// → (wrap back to Intraday).
pub const PERIOD_CYCLE: [ChartPeriod; 5] = [
    ChartPeriod::Intraday,
    ChartPeriod::M1,
    ChartPeriod::M5,
    ChartPeriod::M15,
    ChartPeriod::D1,
];

/// One step forward in [`PERIOD_CYCLE`], wrapping at the end.
pub fn next_period(period: ChartPeriod) -> ChartPeriod {
    let at = PERIOD_CYCLE
        .iter()
        .position(|candidate| *candidate == period)
        .unwrap_or_default();
    PERIOD_CYCLE[(at + 1) % PERIOD_CYCLE.len()]
}

/// The stepper chip's label for a period.
pub fn period_label(period: ChartPeriod) -> &'static str {
    match period {
        ChartPeriod::Intraday => "INTRA",
        ChartPeriod::M1 => "M1",
        ChartPeriod::M5 => "M5",
        ChartPeriod::M15 => "M15",
        ChartPeriod::D1 => "D1",
    }
}

/// Maps one live L2 socket book into the organism's string ladder. The
/// percentage is `spread / mid × 100` at 4-dp (`mid = (best_bid +
/// best_ask) / 2`, guarded non-zero); `None` spread when either ladder is
/// empty, so a half-full socket book falls back cleanly.
pub fn book_view(book: &Level2Book) -> (Vec<BookLevel>, Vec<BookLevel>, Option<SpreadInfo>) {
    if book.bids.is_empty() || book.asks.is_empty() {
        return (Vec::new(), Vec::new(), None);
    }
    let map = |levels: &[BookLevelDto]| {
        levels
            .iter()
            .map(|level| BookLevel {
                price: fixtures::fmt(level.price),
                size: fixtures::fmt(level.size),
                order_count: level.order_count.to_string(),
            })
            .collect()
    };
    let (bids, asks) = (map(&book.bids), map(&book.asks));
    let best_bid = book.bids[0].price;
    let best_ask = book.asks[0].price;
    let spread = best_ask - best_bid;
    let mid = (best_bid + best_ask) / dec!(2);
    let spread_info = if mid == Decimal::ZERO {
        None
    } else {
        Some(SpreadInfo {
            absolute: fixtures::fmt(spread),
            percentage: format!("{}%", (spread * dec!(100) / mid).round_dp(4)),
        })
    };
    (bids, asks, spread_info)
}
