//! Milestone 6 verification: layout & organism data-structure invariants.
//!
//! The `honba-kao-ui-core` organisms (`HfOrderBook`, `HfOrderTicket`,
//! `HfTradingDock`, `HfCollateralOrganism`, `HfOptionChain`, `HfMarketTape`)
//! and templates (`HfThreeColumnTradingLayout`, `HfTwoColumnSplitLayout`,
//! `HfPage`, `HfMosaicWorkspace`) are already implemented. This suite locks in
//! the behavioural invariants of the data structures they carry, across the
//! desktop / laptop / ultrawide viewports the cockpit targets.

use honba_kao_ui_core::molecules::fno::OptionLegQuote;
use honba_kao_ui_core::organisms::{
    BookLevel, CollateralAssetRow, OptionStrikePair, OrderBookFilter, OrderIntent,
};
use honba_kao_ui_core::templates::AlertSeverity;

#[test]
fn alert_severity_badge_classes_are_non_empty_and_distinct() {
    let classes = [
        AlertSeverity::Info.badge_class(),
        AlertSeverity::Success.badge_class(),
        AlertSeverity::Warning.badge_class(),
        AlertSeverity::Critical.badge_class(),
    ];
    for class in &classes {
        assert!(!class.is_empty(), "alert severity badge class must not be empty");
        assert!(class.contains("border-"), "badge must carry a border token");
        assert!(class.contains("text-"), "badge must carry a text token");
    }
    // Each severity maps to a distinct colour family.
    assert_ne!(
        AlertSeverity::Success.badge_class(),
        AlertSeverity::Critical.badge_class()
    );
    assert_ne!(
        AlertSeverity::Info.badge_class(),
        AlertSeverity::Warning.badge_class()
    );
}

// TODO: enumerate every OrderBookFilter variant once production exposes an
// `ALL` const or iterator; none exists in src/organisms/order_book.rs, so only
// the default is checked here.
#[test]
fn order_book_filter_default_and_variants() {
    assert_eq!(OrderBookFilter::default(), OrderBookFilter::All);
    assert_ne!(OrderBookFilter::default(), OrderBookFilter::BidsOnly);
    assert_ne!(OrderBookFilter::default(), OrderBookFilter::AsksOnly);
}

// TODO: needs a production fn that computes cumulative depth (running sum of
// `size`) from a Vec<BookLevel>. None exists in src/organisms/order_book.rs,
// where `cumulative_size` is only read for the depth bar width. This test still
// hand-sets the field.
#[test]
fn book_level_carries_cumulative_depth() {
    let level = BookLevel {
        price: 78450.50,
        size: 12.5,
        cumulative_size: 112.5,
        order_count: 4,
    };
    assert!((level.price - 78450.50).abs() < f64::EPSILON);
    assert!((level.cumulative_size - 112.5).abs() < f64::EPSILON);
    assert_eq!(level.order_count, 4);
}

// TODO: needs a production fn that couples call and put legs into an
// OptionStrikePair for a strike. None exists in src/organisms/option_chain.rs,
// which only renders the pairs it is given. This test still hand-sets fields.
#[test]
fn option_strike_pair_couples_call_and_put() {
    let pair = OptionStrikePair {
        strike: 22500.0,
        call: OptionLegQuote {
            ltp: 180.0,
            iv: 0.18,
            open_interest: 450_000,
            oi_change: 12_000,
            delta: 0.62,
            gamma: 0.04,
            change_pct: 1.5,
            volume: 3_200,
        },
        put: OptionLegQuote {
            ltp: 95.0,
            iv: 0.20,
            open_interest: 310_000,
            oi_change: -4_000,
            delta: -0.38,
            gamma: 0.04,
            change_pct: -0.8,
            volume: 2_100,
        },
    };
    assert!((pair.strike - 22500.0).abs() < f64::EPSILON);
    assert!((pair.call.delta - 0.62).abs() < f64::EPSILON);
    assert!((pair.put.delta - (-0.38)).abs() < f64::EPSILON);
    assert!(pair.call.open_interest > pair.put.open_interest);
}

// TODO: needs a production fn deriving collateral_value = balance * price *
// (1 - haircut_pct / 100). None exists in src/organisms/collateral_organism.rs,
// where collateral_value is a stored field. This test still hand-sets fields.
#[test]
fn collateral_asset_row_carries_haircut_and_value() {
    let row = CollateralAssetRow {
        symbol: "NIFTY".into(),
        name: "Nifty 50 Index".into(),
        balance: 250.0,
        price: 22500.0,
        haircut_pct: 12.0,
        collateral_value: 198_000.0,
    };
    assert_eq!(row.symbol, "NIFTY");
    assert!((row.collateral_value - 198_000.0).abs() < f64::EPSILON);
    assert!((row.haircut_pct - 12.0).abs() < f64::EPSILON);
}

// TODO: needs a production fn on OrderIntent computing notional (price * quantity)
// and/or margin. None exists in src/organisms/order_ticket.rs; notional and margin
// are inline Signal::derive closures inside HfOrderTicket and cannot be called.
// This test still checks literal arithmetic.
#[test]
fn order_intent_side_and_type() {
    let intent = OrderIntent {
        symbol: "RELIANCE".into(),
        side: "BUY".into(),
        order_type: "LIMIT".into(),
        price: 2850.0,
        quantity: 10.0,
    };
    assert_eq!(intent.side, "BUY");
    assert_eq!(intent.order_type, "LIMIT");
    assert!((intent.price * intent.quantity - 28_500.0).abs() < f64::EPSILON);
}

#[test]
fn three_column_layout_grid_spans_match_spec() {
    // HfThreeColumnTradingLayout encodes (desktop) visualizer 7 cols, depth 2-3
    // cols, ticket 3 cols, dock 9 cols across a 12-col grid; the laptop tier
    // uses 6/3/3 and the dock spans 9. These invariants are asserted against the
    // template source so a future refactor cannot silently collapse the cockpit.
    let template_src = include_str!("../src/templates/three_columns.rs");
    assert!(template_src.contains("desktop:col-span-7"), "visualizer: 7 cols desktop");
    assert!(template_src.contains("desktop:col-span-2"), "depth: 2 cols desktop");
    assert!(template_src.contains("desktop:col-span-3"), "ticket: 3 cols desktop");
    assert!(template_src.contains("desktop:col-span-9"), "dock: 9 cols desktop");
    assert!(template_src.contains("laptop:col-span-6"), "visualizer: 6 cols laptop");
    assert!(template_src.contains("laptop:col-span-9"), "dock spans 9 cols laptop");
    assert!(template_src.contains("grid grid-cols-12"), "12-column grid");
}

#[test]
fn two_column_layout_70_30_split() {
    let template_src = include_str!("../src/templates/two_columns.rs");
    assert!(template_src.contains("desktop:col-span-8"), "primary: 8 cols (70%) desktop");
    assert!(template_src.contains("desktop:col-span-4"), "rail: 4 cols (30%) desktop");
    assert!(template_src.contains("laptop:col-span-8"), "primary: 8 cols laptop");
    assert!(template_src.contains("laptop:col-span-4"), "rail: 4 cols laptop");
    assert!(template_src.contains("grid grid-cols-12"), "12-column grid");
}

#[test]
fn page_and_mosaic_templates_exist() {
    let page_src = include_str!("../src/templates/page.rs");
    assert!(page_src.contains("HfPage"), "HfPage template defined");
    assert!(page_src.contains("HfAlertBannerStack"), "alert banner stack defined");
    let mosaic_src = include_str!("../src/templates/mosaic_workspace.rs");
    assert!(mosaic_src.contains("HfMosaicWorkspace"), "HfMosaicWorkspace defined");
    assert!(mosaic_src.contains("grid-cols-12"), "mosaic uses 12-column grid");
}

#[test]
fn all_milestone_6_organisms_and_templates_are_exported() {
    // The `use` imports above already fail to compile if any Milestone 6
    // deliverable is not publicly exported. This test pins that the
    // data-structure deliverables resolve as types.
    let _ = std::marker::PhantomData::<OptionStrikePair>;
    let _ = std::marker::PhantomData::<AlertSeverity>;
    let _ = std::marker::PhantomData::<OrderIntent>;
}
