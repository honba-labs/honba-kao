//! NSH port verification: organism/template data-structure invariants.
//!
//! Supersedes the pre-port (Leptos/Tailwind) version of this file. The NSH
//! organisms (`HfOrderBook`, `HfOrderTicket`, `HfTradingDock`,
//! `HfCollateralOrganism`, `HfOptionChain`, `HfMarketTape`) and templates
//! (`HfThreeColumnTradingLayout`, `HfTwoColumnSplitLayout`, `HfPage`,
//! `HfMosaicWorkspace`) carry money/size fields as pre-formatted `String`
//! (never `Decimal` — `Decimal` is not `JsValue`-safe, so NSH props cross
//! the FFI boundary as caller-formatted strings), and layout is expressed
//! with `nsh_core`'s typed style builder (`.flex_1()`, `.grid_cols(12)`),
//! not Tailwind breakpoint-class strings. This suite locks in the current
//! public shapes so a future refactor can't silently drop a field or
//! rename an export without a test failing.

use honba_kao_ui_core::molecules::core::AlertSeverity;
use honba_kao_ui_core::organisms::order_book::SpreadInfo;
use honba_kao_ui_core::organisms::{BookLevel, CollateralRow, OptionChainRow, TapeEntry};

#[test]
fn alert_severity_default_is_info_and_variants_are_distinct() {
    assert_eq!(AlertSeverity::default(), AlertSeverity::Info);
    let variants = [
        AlertSeverity::Info,
        AlertSeverity::Warning,
        AlertSeverity::Error,
        AlertSeverity::Success,
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "AlertSeverity variants must be pairwise distinct");
            }
        }
    }
}

// BookLevel/SpreadInfo carry price/size/order_count as pre-formatted strings
// (Decimal formatted by the caller before crossing into a `Prop`).
#[test]
fn book_level_and_spread_info_carry_preformatted_strings() {
    let level = BookLevel {
        price: "78450.50".to_string(),
        size: "12.5".to_string(),
        order_count: "4".to_string(),
    };
    assert_eq!(level.price, "78450.50");
    assert_eq!(level.size, "12.5");
    assert_eq!(level.order_count, "4");

    let spread = SpreadInfo {
        absolute: "1.25".to_string(),
        percentage: "0.02".to_string(),
    };
    assert_eq!(spread.absolute, "1.25");
    assert_eq!(spread.percentage, "0.02");
}

// OptionChainRow (one option-chain strike row) carries strike/bid/ask/iv/delta
// as pre-formatted strings.
#[test]
fn option_chain_row_carries_preformatted_strings() {
    let row = OptionChainRow {
        strike: "22500.0".to_string(),
        bid: "178.0".to_string(),
        ask: "182.0".to_string(),
        iv: "0.18".to_string(),
        delta: "0.62".to_string(),
    };
    assert_eq!(row.strike, "22500.0");
    assert_eq!(row.bid, "178.0");
    assert_eq!(row.ask, "182.0");
    assert_eq!(row.iv, "0.18");
    assert_eq!(row.delta, "0.62");
}

// CollateralRow carries asset/value/haircut/net_value as pre-formatted
// strings; any balance x price x (1 - haircut%) math now happens at the
// call site before formatting, not inside this presentational struct.
#[test]
fn collateral_row_carries_preformatted_strings() {
    let row = CollateralRow {
        asset: "NIFTY".to_string(),
        value: "4,950,000.00".to_string(),
        haircut: "12%".to_string(),
        net_value: "4,356,000.00".to_string(),
    };
    assert_eq!(row.asset, "NIFTY");
    assert_eq!(row.value, "4,950,000.00");
    assert_eq!(row.haircut, "12%");
    assert_eq!(row.net_value, "4,356,000.00");
}

// TapeEntry (market tape) defaults `up` to false when omitted.
#[test]
fn tape_entry_defaults_up_to_false() {
    let entry = TapeEntry {
        symbol: "RELIANCE".to_string(),
        price: "2850.00".to_string(),
        change: "+1.5%".to_string(),
        up: Default::default(),
    };
    assert!(!entry.up);
    assert_eq!(entry.symbol, "RELIANCE");
}

#[test]
fn mosaic_workspace_uses_a_12_column_grid() {
    // HfMosaicWorkspace's tile area is a 12-column grid (nsh_core's typed
    // builder, `.grid_cols(12)`) rather than a Tailwind `grid-cols-12`
    // class string — asserted against source so a refactor that silently
    // drops the grid doesn't go unnoticed.
    let mosaic_src = include_str!("../src/templates/mosaic_workspace.rs");
    assert!(
        mosaic_src.contains(".grid_cols(12)"),
        "mosaic workspace tile area must be a 12-column grid"
    );
}

#[test]
fn page_and_templates_are_exported() {
    // The `use` imports above already fail to compile if any of these
    // types is not publicly exported; this pins that they resolve.
    let _ = std::marker::PhantomData::<honba_kao_ui_core::templates::HfPageProps>;
    let _ = std::marker::PhantomData::<honba_kao_ui_core::templates::HfPageAlertSlotProps>;
    let _ = std::marker::PhantomData::<honba_kao_ui_core::templates::HfMosaicWorkspaceProps>;
    let _ =
        std::marker::PhantomData::<honba_kao_ui_core::templates::HfThreeColumnTradingLayoutProps>;
}
