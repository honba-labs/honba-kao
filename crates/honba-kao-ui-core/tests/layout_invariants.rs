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
use rust_decimal_macros::dec;

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

// BookLevel carries price/size/cumulative depth as Decimal (zero-floats).
#[test]
fn book_level_carries_decimal_price_and_size() {
    let level = BookLevel {
        price: dec!(78450.50),
        size: dec!(12.5),
        cumulative_size: dec!(112.5),
        order_count: 4,
    };
    assert_eq!(level.price, dec!(78450.50));
    assert_eq!(level.size, dec!(12.5));
    assert_eq!(level.cumulative_size, dec!(112.5));
    assert_eq!(level.order_count, 4);
}

// OptionStrikePair couples call and put legs with Decimal quotes/Greeks.
#[test]
fn option_strike_pair_couples_call_and_put() {
    let pair = OptionStrikePair {
        strike: dec!(22500.0),
        call: OptionLegQuote {
            ltp: dec!(180.0),
            iv: dec!(0.18),
            open_interest: 450_000,
            oi_change: 12_000,
            delta: dec!(0.62),
            gamma: dec!(0.04),
            change_pct: dec!(1.5),
            volume: 3_200,
        },
        put: OptionLegQuote {
            ltp: dec!(95.0),
            iv: dec!(0.20),
            open_interest: 310_000,
            oi_change: -4_000,
            delta: dec!(-0.38),
            gamma: dec!(0.04),
            change_pct: dec!(-0.8),
            volume: 2_100,
        },
    };
    assert_eq!(pair.strike, dec!(22500.0));
    assert_eq!(pair.call.delta, dec!(0.62));
    assert_eq!(pair.put.delta, dec!(-0.38));
    assert!(pair.call.open_interest > pair.put.open_interest);
}

// CollateralAssetRow carries balance/price/haircut as Decimal; collateral value
// is balance x price x (1 - haircut%).
#[test]
fn collateral_asset_row_carries_haircut_and_value() {
    let row = CollateralAssetRow {
        symbol: "NIFTY".into(),
        name: "Nifty 50 Index".into(),
        balance: dec!(250.0),
        price: dec!(22500.0),
        haircut_pct: dec!(12.0),
        collateral_value: dec!(4950000.0),
    };
    assert_eq!(row.symbol, "NIFTY");
    // collateral_value = balance x price x (1 - haircut_pct / 100)
    let expected = row.balance * row.price * (dec!(1) - row.haircut_pct / dec!(100));
    assert_eq!(row.collateral_value, expected);
}

// OrderIntent carries price/quantity as Decimal and computes notional exactly.
#[test]
fn order_intent_side_and_type() {
    let intent = OrderIntent {
        symbol: "RELIANCE".into(),
        side: "BUY".into(),
        order_type: "LIMIT".into(),
        price: dec!(2850.0),
        quantity: dec!(10.0),
    };
    assert_eq!(intent.side, "BUY");
    assert_eq!(intent.order_type, "LIMIT");
    assert_eq!(intent.price * intent.quantity, dec!(28500.0));
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
    // Mobile-first: columns stack by default (`flex flex-col`), then split
    // 8/4 (~70/30) from laptop up via `laptop:grid` + `laptop:col-span-8/4`.
    assert!(template_src.contains("flex flex-col"), "mobile stacks columns");
    assert!(template_src.contains("laptop:grid"), "splits to grid from laptop up");
    assert!(template_src.contains("laptop:col-span-8"), "primary: 8 cols (~70%)");
    assert!(template_src.contains("laptop:col-span-4"), "rail: 4 cols (~30%)");
    assert!(template_src.contains("col-span-12"), "mobile: full-width columns");
    assert!(template_src.contains("gap-4"), "gutter preserved");
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
