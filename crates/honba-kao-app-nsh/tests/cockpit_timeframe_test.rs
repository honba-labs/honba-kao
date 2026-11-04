//! Trading Cockpit timeframe stepper + depth-mapping tests: the pure
//! logic behind the cockpit's chart-period stepper cycle and the L2
//! socket book mapper. All financial assertions are hand-derived from
//! the deterministic fixture walk (drift `((i*37) % 43) − 21`), which
//! by contract is regenerated in `Decimal`, never parsed back from
//! strings.

use honba_kao_app_nsh::features::trading::cockpit_page::{
    PERIOD_CYCLE, book_view, candles_for_period, next_period, period_label,
};
use honba_kao_client::dto::market::{BookLevelDto, Level2Book};
use honba_kao_ui_core::organisms::candlestick_chart::ChartPeriod;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

#[test]
fn the_cycle_visits_all_five_periods_and_wraps() {
    let mut seen = Vec::new();
    let mut current = ChartPeriod::M5;
    for _ in 0..PERIOD_CYCLE.len() {
        seen.push(current);
        current = next_period(current);
    }
    assert_eq!(current, ChartPeriod::M5, "five steps from M5 return to M5");
    // Every cycle entry was visited exactly once, rotation-normalised to
    // the array order (the walk starts mid-cycle at M5).
    let head = PERIOD_CYCLE
        .iter()
        .position(|candidate| *candidate == ChartPeriod::M5)
        .unwrap_or_default();
    let rotated: Vec<ChartPeriod> = PERIOD_CYCLE
        .iter()
        .cycle()
        .skip(head)
        .take(PERIOD_CYCLE.len())
        .copied()
        .collect();
    assert_eq!(seen, rotated);
}

#[test]
fn the_specific_transitions_wrap_at_both_ends() {
    assert_eq!(next_period(ChartPeriod::M15), ChartPeriod::D1);
    assert_eq!(next_period(ChartPeriod::D1), ChartPeriod::Intraday);
    assert_eq!(next_period(ChartPeriod::Intraday), ChartPeriod::M1);
}

#[test]
fn period_labels_match_the_chip_names() {
    assert_eq!(period_label(ChartPeriod::Intraday), "INTRA");
    assert_eq!(period_label(ChartPeriod::M1), "M1");
    assert_eq!(period_label(ChartPeriod::M5), "M5");
    assert_eq!(period_label(ChartPeriod::M15), "M15");
    assert_eq!(period_label(ChartPeriod::D1), "D1");
}

#[test]
fn the_one_to_one_periods_keep_the_full_walk() {
    let base = dec!(22842.50);
    assert_eq!(candles_for_period(base, ChartPeriod::Intraday).len(), 60);
    assert_eq!(candles_for_period(base, ChartPeriod::M1).len(), 60);
    assert_eq!(candles_for_period(base, ChartPeriod::M5).len(), 60);
}

#[test]
fn m15_merges_three_sources_per_candle() {
    let base = dec!(22842.50);
    let merged = candles_for_period(base, ChartPeriod::M15);
    assert_eq!(merged.len(), 20, "60 sources at merge 3");
    // Group g's first source is i = 3g; the group keeps that source's
    // 6-minute label: g = 0 → 09:15, g = 19 → i = 57 → minute 897 → 14:57.
    assert_eq!(merged[0].label, "09:15");
    assert_eq!(merged[19].label, "14:57");
    // Take-last close: the last merged candle is source 59's close,
    // drift(59) = (59*37) % 43 - 21 = 12.
    assert_eq!(merged[19].close, "22854.50");
}

#[test]
fn d1_merges_six_sources_per_candle() {
    let base = dec!(22842.50);
    let merged = candles_for_period(base, ChartPeriod::D1);
    assert_eq!(merged.len(), 10, "60 sources at merge 6");
    // Day candles are relabelled D1..D10.
    for (i, candle) in merged.iter().enumerate() {
        assert_eq!(candle.label, format!("D{}", i + 1));
    }
    // Take-last close of the first group: source 5, drift(5) = -8.
    assert_eq!(merged[0].close, "22834.50");
}

#[test]
fn the_resample_is_deterministic_and_decimal_custodied() {
    let base = dec!(22842.50);
    assert_eq!(
        candles_for_period(base, ChartPeriod::M15),
        candles_for_period(base, ChartPeriod::M15),
        "identical calls produce identical candles"
    );
    for period in PERIOD_CYCLE {
        for candle in candles_for_period(base, period) {
            assert!(
                Decimal::from_str(&candle.ma5).is_ok(),
                "ma5 parses: {}",
                candle.ma5
            );
        }
    }
}

fn level(price_cents: i64, size_hundredths: i64, order_count: u32) -> BookLevelDto {
    BookLevelDto {
        price: Decimal::new(price_cents, 2),
        size: Decimal::new(size_hundredths, 2),
        order_count,
    }
}

fn sample_book() -> Level2Book {
    Level2Book {
        symbol: "RELIANCE".to_string(),
        bids: vec![level(12_700, 1_000, 4), level(12_600, 500, 3)],
        asks: vec![level(12_800, 800, 2), level(12_900, 600, 1)],
        timestamp: 0,
    }
}

#[test]
fn book_view_maps_the_socket_ladder_with_a_centred_spread() {
    let (bids, asks, spread) = book_view(&sample_book());
    assert_eq!(bids[0].price, "127.00");
    assert_eq!(bids[0].size, "10.00");
    assert_eq!(bids[0].order_count, "4");
    assert_eq!(asks[0].price, "128.00");
    assert_eq!(asks[0].size, "8.00");
    assert_eq!(asks[1].order_count, "1");
    let spread = spread.expect("a full ladder carries a spread");
    // absolute: 128.00 - 127.00 → "1.00".
    assert_eq!(spread.absolute, "1.00");
    // percentage: 1.00 x 100 / ((127.00 + 128.00) / 2) at 4 dp → 0.7843%.
    assert_eq!(spread.percentage, "0.7843%");
}

#[test]
fn book_view_falls_back_to_none_on_a_half_empty_socket() {
    let empty = Level2Book {
        symbol: "RELIANCE".to_string(),
        bids: vec![],
        asks: vec![],
        timestamp: 0,
    };
    let (bids, asks, spread) = book_view(&empty);
    assert!(bids.is_empty());
    assert!(asks.is_empty());
    assert!(spread.is_none());
}
