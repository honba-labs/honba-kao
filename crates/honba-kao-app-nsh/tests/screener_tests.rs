//! Pure-logic host tests for the screener's public API:
//! `table_grid`'s filter/sort/export pipeline and `types`' Indian
//! notation helpers. No rendering, no I/O, no floats.

use honba_kao_app_nsh::features::screener::table_grid::{
    default_visible_columns, export_csv, filter_instruments, screener_universe, sort_instruments,
};
use honba_kao_app_nsh::features::screener::types::{
    CSV_COLUMNS, FilterCriteria, MarketCapTier, QuickPreset, ScreenerInstrument, ScreenerViewTab,
    SortDirection, SortState, compact_cap_crore, compact_volume, inr, percent_text, signed_inr,
    tab_columns,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Asserts a preset keeps exactly the universe rows satisfying `keep`
/// (value-predicate membership only; never ordering).
fn assert_preset_membership(
    universe: &[ScreenerInstrument],
    preset: QuickPreset,
    keep: impl Fn(&ScreenerInstrument) -> bool,
) {
    let rows = filter_instruments(universe, preset, "", &FilterCriteria::default(), &[]);
    for row in &rows {
        assert!(keep(row), "{preset:?} kept non-matching row {}", row.symbol);
    }
    for inst in universe {
        let expected = keep(inst);
        let got = rows.iter().any(|r| r.symbol == inst.symbol);
        assert_eq!(got, expected, "{preset:?} membership for {}", inst.symbol);
    }
}

/// Counts CSV fields, honoring quoting (a quoted cell may contain commas).
fn csv_field_count(line: &str) -> usize {
    let mut fields = 1;
    let mut in_quotes = false;
    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => fields += 1,
            _ => {}
        }
    }
    fields
}

#[test]
fn universe_invariants() {
    let universe = screener_universe();
    assert_eq!(universe.len(), 30);
    let mut symbols = std::collections::HashSet::new();
    for inst in &universe {
        assert!(
            symbols.insert(inst.symbol.clone()),
            "duplicate symbol {}",
            inst.symbol
        );
        assert!(inst.price > Decimal::ZERO, "{} price", inst.symbol);
        assert!(inst.market_cap_cr > Decimal::ZERO, "{} cap", inst.symbol);
        assert!(inst.high_52 > inst.price, "{} above 52W high", inst.symbol);
        assert!(inst.price > inst.low_52, "{} below 52W low", inst.symbol);
        assert!(
            inst.rsi_14 >= Decimal::ZERO && inst.rsi_14 <= Decimal::from(100),
            "{} rsi {}",
            inst.symbol,
            inst.rsi_14
        );
        assert!(inst.volume >= 0, "{} volume", inst.symbol);
        assert!(inst.technical_rating.rank() <= 4);
    }
}

#[test]
fn default_criteria_passes_everything() {
    let universe = screener_universe();
    let rows = filter_instruments(
        &universe,
        QuickPreset::All,
        "",
        &FilterCriteria::default(),
        &[],
    );
    assert_eq!(rows, universe);
}

#[test]
fn preset_predicates() {
    let universe = screener_universe();

    assert_preset_membership(&universe, QuickPreset::Gainers, |inst| {
        inst.change_percent > Decimal::ZERO
    });
    assert_preset_membership(&universe, QuickPreset::Losers, |inst| {
        inst.change_percent < Decimal::ZERO
    });
    // Value also requires a strictly positive P/E; `None` P/E drops.
    assert_preset_membership(
        &universe,
        QuickPreset::Value,
        |inst| matches!(inst.pe_ratio, Some(pe) if pe > Decimal::ZERO && pe < dec!(25)),
    );
    // Source pins `>= 1.5` (the label reads ">1.5"); `None` drops.
    assert_preset_membership(
        &universe,
        QuickPreset::Dividend,
        |inst| matches!(inst.dividend_yield, Some(y) if y >= dec!(1.5)),
    );
    assert_preset_membership(&universe, QuickPreset::Oversold, |inst| {
        inst.rsi_14 < dec!(35)
    });
    assert_preset_membership(&universe, QuickPreset::Overbought, |inst| {
        inst.rsi_14 > dec!(70)
    });
}

#[test]
fn criteria_bounds_respected() {
    let universe = screener_universe();

    let price_floor = FilterCriteria {
        sector: String::new(),
        market_cap_tier: MarketCapTier::All,
        min_price: Some(dec!(1000)),
        ..Default::default()
    };
    let rows = filter_instruments(&universe, QuickPreset::All, "", &price_floor, &[]);
    for row in &rows {
        assert!(row.price >= dec!(1000), "{} below floor", row.symbol);
    }
    let expected: Vec<ScreenerInstrument> = universe
        .iter()
        .filter(|inst| inst.price >= dec!(1000))
        .cloned()
        .collect();
    assert_eq!(rows, expected);

    let pe_cap = FilterCriteria {
        max_pe: Some(dec!(20)),
        ..Default::default()
    };
    let pe_rows = filter_instruments(&universe, QuickPreset::All, "", &pe_cap, &[]);
    for row in &pe_rows {
        // `None`-P/E rows are excluded when a P/E bound is active.
        let pe = row.pe_ratio.expect("None P/E row passed a max_pe bound");
        assert!(pe <= dec!(20), "{} P/E {pe}", row.symbol);
    }
    let expected: Vec<ScreenerInstrument> = universe
        .iter()
        .filter(|inst| matches!(inst.pe_ratio, Some(pe) if pe <= dec!(20)))
        .cloned()
        .collect();
    assert_eq!(pe_rows, expected);
}

#[test]
fn search_matches_symbol_or_name() {
    let universe = screener_universe();

    let all = filter_instruments(
        &universe,
        QuickPreset::All,
        "",
        &FilterCriteria::default(),
        &[],
    );
    assert_eq!(all, universe);

    // Fixture-agnostic needle: symbol/name/sector, case-insensitive.
    let needle = "a";
    let rows = filter_instruments(
        &universe,
        QuickPreset::All,
        needle,
        &FilterCriteria::default(),
        &[],
    );
    assert!(!rows.is_empty(), "needle {needle:?} should match rows");
    let q = needle.to_lowercase();
    for row in &rows {
        assert!(
            row.symbol.to_lowercase().contains(&q)
                || row.name.to_lowercase().contains(&q)
                || row.sector.to_lowercase().contains(&q),
            "{} matched without symbol/name/sector",
            row.symbol
        );
    }
    for inst in &universe {
        let matches = inst.symbol.to_lowercase().contains(&q)
            || inst.name.to_lowercase().contains(&q)
            || inst.sector.to_lowercase().contains(&q);
        assert_eq!(
            rows.iter().any(|r| r.symbol == inst.symbol),
            matches,
            "membership for {}",
            inst.symbol
        );
    }

    let upper = filter_instruments(
        &universe,
        QuickPreset::All,
        "A",
        &FilterCriteria::default(),
        &[],
    );
    assert_eq!(rows, upper, "search must be case-insensitive");
}

#[test]
fn sort_states() {
    let universe = screener_universe();

    let desc = SortState {
        column_id: "market_cap_cr".into(),
        direction: SortDirection::Desc,
    };
    let sorted = sort_instruments(universe.clone(), Some(&desc));
    let top = sorted
        .first()
        .expect("sorted board is non-empty")
        .market_cap_cr;
    for row in &sorted {
        assert!(
            top >= row.market_cap_cr,
            "{} above the Desc top",
            row.symbol
        );
    }

    let asc = SortState {
        column_id: "market_cap_cr".into(),
        direction: SortDirection::Asc,
    };
    let sorted_asc = sort_instruments(universe.clone(), Some(&asc));
    let bottom = sorted_asc
        .first()
        .expect("sorted board is non-empty")
        .market_cap_cr;
    for row in &sorted_asc {
        assert!(
            bottom <= row.market_cap_cr,
            "{} below the Asc top",
            row.symbol
        );
    }

    let kept = sort_instruments(universe.clone(), None);
    assert_eq!(kept, universe, "None sort keeps the original order");
}

#[test]
fn tri_state_direction_cycle() {
    assert_eq!(SortDirection::first(), SortDirection::Desc);
    assert_eq!(SortDirection::Desc.next(), Some(SortDirection::Asc));
    assert_eq!(SortDirection::Asc.next(), None);
}

#[test]
fn export_csv_shape() {
    let universe = screener_universe();
    let csv = export_csv(&universe);
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 31, "header + 30 rows");
    assert_eq!(lines[0], CSV_COLUMNS.join(","));
    assert_eq!(CSV_COLUMNS.len(), 16);
    for line in &lines {
        assert_eq!(csv_field_count(line), 16, "line: {line}");
    }
}

#[test]
fn indian_notation_helpers() {
    assert_eq!(inr(dec!(2842.50)), "₹2,842.50");
    assert_eq!(inr(dec!(-15.60)), "-₹15.60");
    assert_eq!(inr(dec!(0)), "₹0.00");

    assert_eq!(signed_inr(dec!(52.30)), "+₹52.30");
    assert_eq!(signed_inr(dec!(0)), "₹0.00");

    assert_eq!(percent_text(dec!(1.84)), "+1.84%");
    assert_eq!(percent_text(dec!(-0.40)), "-0.40%");
    assert_eq!(percent_text(dec!(0)), "0.00%");

    assert_eq!(compact_volume(42_000_000), "4.2Cr");
    assert_eq!(compact_volume(420_000), "4.2L");
    assert_eq!(compact_volume(8_400), "8,400");
    assert_eq!(compact_volume(999_999), "10L");

    assert_eq!(compact_cap_crore(dec!(1_920_000)), "₹19.2L Cr");
    assert_eq!(compact_cap_crore(dec!(45_000)), "₹45,000 Cr");
}

#[test]
fn active_count_and_tiers() {
    assert_eq!(FilterCriteria::default().active_count(), 0);

    let criteria = FilterCriteria {
        sector: "Banking".into(),
        market_cap_tier: MarketCapTier::Mega,
        min_price: Some(dec!(1000)),
        ..Default::default()
    };
    assert_eq!(criteria.active_count(), 3);

    assert!(MarketCapTier::Mega.contains(dec!(200_000)));
    assert!(MarketCapTier::Large.contains(dec!(199_999)));
    assert!(MarketCapTier::Large.contains(dec!(50_000)));
    assert!(MarketCapTier::Small.contains(dec!(9_999)));
}

#[test]
fn default_visible_columns_matches_tab() {
    for tab in ScreenerViewTab::ALL {
        let expected: Vec<String> = tab_columns(*tab).iter().map(|c| c.id.to_string()).collect();
        assert_eq!(default_visible_columns(*tab), expected, "tab {tab:?}");
    }
}
