//! Heatmap view for the screener: market-cap weighted tiles colored by % change.
//!
//! Doc §5.1 grading. All grading math is exact `Decimal` (zero-float invariant);
//! `f32` appears only at the layout-px boundary inside [`tile_size_pts`].

use crate::features::screener::types::{ScreenerInstrument, compact_cap_crore, percent_text};
use honba_kao_ui_core::theme::tokens::{Finish, Finished, HonbaStyle, MONO, colors, radius};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Screener layout toggle owned by the page; this module only defines it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenerViewMode {
    /// Default dense table grid.
    #[default]
    Table,
    /// Market-cap heatmap tiles.
    Heatmap,
}

impl ScreenerViewMode {
    /// Canonical toggle order.
    pub const ALL: &'static [ScreenerViewMode] =
        &[ScreenerViewMode::Table, ScreenerViewMode::Heatmap];

    /// Short label for the view-mode toggle.
    pub fn label(self) -> &'static str {
        match self {
            ScreenerViewMode::Table => "Table",
            ScreenerViewMode::Heatmap => "Heatmap",
        }
    }
}

/// Heat fill for a % change bucket (doc §5.1). Dark-surface-first: the same hex
/// is used for light and dark themes so the grading stays byte-identical.
pub fn heat_color(change_percent: Decimal) -> AdaptiveColor {
    if change_percent >= dec!(3.0) {
        AdaptiveColor::rgb(0x089981, 0x089981)
    } else if change_percent >= dec!(1.5) {
        AdaptiveColor::rgb(0x0e7a63, 0x0e7a63)
    } else if change_percent > dec!(0.0) {
        AdaptiveColor::rgb(0x105949, 0x105949)
    } else if change_percent == dec!(0.0) {
        AdaptiveColor::rgb(0x2a2e39, 0x2a2e39)
    } else if change_percent > dec!(-1.5) {
        AdaptiveColor::rgb(0x6b2029, 0x6b2029)
    } else if change_percent > dec!(-3.0) {
        AdaptiveColor::rgb(0x992430, 0x992430)
    } else {
        AdaptiveColor::rgb(0xf23645, 0xf23645)
    }
}

/// Treemap tile width in points, binned by market cap (layout-only `f32`).
pub fn tile_size_pts(market_cap_cr: Decimal) -> f32 {
    if market_cap_cr >= dec!(200_000) {
        140.0
    } else if market_cap_cr >= dec!(50_000) {
        100.0
    } else {
        75.0
    }
}

/// Near-white tile ink: every heat fill is dark, so text keeps contrast on both themes.
const TILE_TEXT: AdaptiveColor = AdaptiveColor::rgb(0xf8fafc, 0xf8fafc);

/// One heatmap tile: fixed size, heat fill, symbol + signed change, tap opens the drawer.
fn heat_tile(
    inst: &ScreenerInstrument,
    active_signal: Signal<String>,
    drawer_signal: Signal<bool>,
) -> View {
    let sym = inst.symbol.clone();
    let sym_label = sym.clone();
    let pct = inst.change_percent;
    let arrow = if inst.change_up() { "▲" } else { "▼" };
    let change_label = format!("{} {}", arrow, percent_text(pct));
    let size = tile_size_pts(inst.market_cap_cr);

    view()
        .w(px(size))
        .h(px(64.0))
        .rounded(px(6.0))
        .bg(heat_color(pct))
        .flex_shrink_0()
        .overflow_hidden()
        .p(px(6.0))
        .flex_col()
        .child(
            text(sym_label)
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .font_family(MONO)
                .text_color(TILE_TEXT),
        )
        .child(text(change_label).text_size(px(10.0)).text_color(TILE_TEXT))
        .on_press(move || {
            active_signal.set(sym.clone());
            drawer_signal.set(true);
        })
}

/// Sector (or "All Market") card: chrome-matched header over a wrapping tile body.
fn heat_card(title: String, total: Decimal, tiles: Vec<View>) -> View {
    let header = view()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .child(
            text(title)
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_muted(),
        )
        .child(
            text(compact_cap_crore(total))
                .text_size(px(10.0))
                .font_family(MONO)
                .text_muted(),
        );

    let mut body = view().flex_row().flex_wrap().gap(px(4.0));
    for tile in tiles {
        body = body.child(tile);
    }

    view()
        .bg(colors::SURFACE)
        .finish(Finish::Card)
        .rounded(px(radius::INSET))
        .border_1()
        .border_color(colors::BORDER)
        .p(px(8.0))
        .flex_col()
        .gap(px(6.0))
        .child(header)
        .child(body)
}

/// Heatmap body: tiles grouped by sector (largest cap first) or as one "All Market" card.
#[allow(non_snake_case)]
pub fn ScreenerHeatmap(
    instruments: Signal<Vec<ScreenerInstrument>>,
    active_symbol: Signal<String>,
    drawer_open: Signal<bool>,
    group_by_sector: Signal<bool>,
) -> View {
    let active_signal = active_symbol;
    let drawer_signal = drawer_open;
    let instruments = instruments.get();
    let group = group_by_sector.get();

    let root = view().flex_col().w_full().h_full().gap(px(6.0)).p(px(8.0));

    if instruments.is_empty() {
        let empty = view()
            .flex_1()
            .items_center()
            .justify_center()
            .child(text("No instruments match the active screen").text_muted());
        return root.child(empty);
    }

    let total_all: Decimal = instruments.iter().map(|i| i.market_cap_cr).sum();

    // Sector totals, ordered by aggregate market cap (desc).
    let mut sectors: Vec<(String, Decimal)> = Vec::new();
    if group {
        for inst in instruments.iter() {
            if let Some(idx) = sectors.iter().position(|(name, _)| *name == inst.sector) {
                sectors[idx].1 += inst.market_cap_cr;
            } else {
                sectors.push((inst.sector.clone(), inst.market_cap_cr));
            }
        }
        sectors.sort_by_key(|b| std::cmp::Reverse(b.1));
    } else {
        sectors.push(("All Market".to_string(), total_all));
    }

    let mut cards: Vec<View> = Vec::new();
    for (title, total) in sectors {
        let mut members: Vec<&ScreenerInstrument> = if group {
            instruments.iter().filter(|i| i.sector == title).collect()
        } else {
            instruments.iter().collect()
        };
        members.sort_by_key(|b| std::cmp::Reverse(b.market_cap_cr));

        let mut tiles: Vec<View> = Vec::new();
        for inst in members {
            tiles.push(heat_tile(inst, active_signal, drawer_signal));
        }
        cards.push(heat_card(title, total, tiles));
    }

    let mut list = scroll().flex_1().flex_col().gap(px(6.0));
    for card in cards {
        list = list.child(card);
    }

    root.child(list)
}
