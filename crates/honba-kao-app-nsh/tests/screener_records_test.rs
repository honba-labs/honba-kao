//! Pure-logic host tests for the multi-asset record layer
//! (screener-design-doc.md §4 / Phase 3 gate): universes, class-aware
//! filtering, record sorting keys, per-class columns, and the segmented
//! CSV export. No rendering, no I/O, no floats.

use honba_kao_app_nsh::features::screener::screener_heatmap::{ScreenerViewMode, tile_size_pts};
use honba_kao_app_nsh::features::screener::table_grid::{
    asset_columns, default_visible_columns, default_visible_columns_for, export_records_csv,
    filter_instruments, filter_records, screener_records, screener_universe, sort_records,
    sort_value_record,
};
use honba_kao_app_nsh::features::screener::types::{
    FilterCriteria, QuickPreset, ScreenerAssetType, ScreenerRecord, SortDirection, SortState,
    compact_cr, tab_columns,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn by_class(records: &[ScreenerRecord], asset: ScreenerAssetType) -> Vec<&ScreenerRecord> {
    records.iter().filter(|r| r.class() == asset).collect()
}

#[test]
fn records_universe_shape() {
    let records = screener_records();
    assert_eq!(records.len(), 60, "30 stocks + 10 ETFs + 10 bonds + 10 MFs");

    let mut symbols = std::collections::HashSet::new();
    for record in &records {
        assert!(
            symbols.insert(record.symbol().to_string()),
            "duplicate record symbol {}",
            record.symbol()
        );
        assert!(
            record.price() > Decimal::ZERO,
            "{} has a price",
            record.symbol()
        );
    }
    assert_eq!(by_class(&records, ScreenerAssetType::Stocks).len(), 30);
    assert_eq!(by_class(&records, ScreenerAssetType::Etf).len(), 10);
    assert_eq!(by_class(&records, ScreenerAssetType::Bonds).len(), 10);
    assert_eq!(by_class(&records, ScreenerAssetType::MutualFunds).len(), 10);
}

#[test]
fn filter_records_by_class() {
    let records = screener_records();
    for asset in ScreenerAssetType::ALL {
        let expected = match asset {
            ScreenerAssetType::Stocks => 30,
            _ => 10,
        };
        let rows = filter_records(
            &records,
            *asset,
            QuickPreset::All,
            "",
            &FilterCriteria::default(),
            &[],
        );
        assert_eq!(rows.len(), expected, "asset {asset:?}");
        assert!(
            rows.iter().all(|r| r.class() == *asset),
            "{asset:?} universe leaked another class"
        );
    }
}

#[test]
fn filter_records_search_is_case_insensitive() {
    let records = screener_records();
    let rows = filter_records(
        &records,
        ScreenerAssetType::Etf,
        QuickPreset::All,
        "niftybees",
        &FilterCriteria::default(),
        &[],
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].symbol(), "NIFTYBEES");

    let mf = filter_records(
        &records,
        ScreenerAssetType::MutualFunds,
        QuickPreset::All,
        "FLEXI",
        &FilterCriteria::default(),
        &[],
    );
    assert!(
        !mf.is_empty(),
        "'FLEXI' reaches flexi-cap schemes' names or AMCs"
    );
}

#[test]
fn filter_records_preset_gainers_losers() {
    let records = screener_records();
    let rows = filter_records(
        &records,
        ScreenerAssetType::Etf,
        QuickPreset::Gainers,
        "",
        &FilterCriteria::default(),
        &[],
    );
    for row in &rows {
        assert!(
            row.change_percent() > Decimal::ZERO,
            "Gainers kept {}",
            row.symbol()
        );
    }
    let losers = filter_records(
        &records,
        ScreenerAssetType::Bonds,
        QuickPreset::Losers,
        "",
        &FilterCriteria::default(),
        &[],
    );
    for row in &losers {
        assert!(
            row.change_percent() < Decimal::ZERO,
            "Losers kept {}",
            row.symbol()
        );
    }
}

#[test]
fn filter_records_stock_only_presets_drop_non_stocks() {
    let records = screener_records();
    for preset in [
        QuickPreset::Value,
        QuickPreset::Dividend,
        QuickPreset::Oversold,
        QuickPreset::Momentum,
        QuickPreset::High52,
        QuickPreset::MostCapitalized,
    ] {
        for asset in [
            ScreenerAssetType::Etf,
            ScreenerAssetType::Bonds,
            ScreenerAssetType::MutualFunds,
        ] {
            let rows = filter_records(&records, asset, preset, "", &FilterCriteria::default(), &[]);
            assert!(
                rows.is_empty(),
                "{preset:?} is stock-only; non-stocks must drop ({asset:?})"
            );
        }
    }
}

#[test]
fn filter_records_shared_price_bound() {
    let records = screener_records();
    let criteria = FilterCriteria {
        min_price: Some(dec!(100.0)),
        ..Default::default()
    };
    let rows = filter_records(
        &records,
        ScreenerAssetType::Etf,
        QuickPreset::All,
        "",
        &criteria,
        &[],
    );
    for row in &rows {
        assert!(
            row.price() >= dec!(100.0),
            "{} below the ₹100 nav floor",
            row.symbol()
        );
    }
}

#[test]
fn filter_records_watchlist() {
    let records = screener_records();
    let rows = filter_records(
        &records,
        ScreenerAssetType::MutualFunds,
        QuickPreset::Watchlist,
        "",
        &FilterCriteria::default(),
        &["PPFCAP".to_string()],
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].symbol(), "PPFCAP");
}

#[test]
fn sort_records_by_aum_desc() {
    let records = screener_records();
    let etfs: Vec<ScreenerRecord> = by_class(&records, ScreenerAssetType::Etf)
        .into_iter()
        .cloned()
        .collect();
    let sort = SortState {
        column_id: "aum_cr".into(),
        direction: SortDirection::Desc,
    };
    let sorted = sort_records(etfs.clone(), Some(&sort));
    let top = sorted[0].price();
    assert!(
        sorted.iter().all(|r| r.symbol() != ""), // shape sanity, ordering below
        "sorted rows are records"
    );
    // The AUM ordering: no earlier row has a smaller AUM than a later one.
    let aum_of = |r: &ScreenerRecord| match r {
        ScreenerRecord::Etf(e) => e.aum_cr,
        _ => dec!(0),
    };
    for window in sorted.windows(2) {
        assert!(
            aum_of(&window[0]) >= aum_of(&window[1]),
            "AUM not descending: {} vs {}",
            window[0].symbol(),
            window[1].symbol()
        );
    }
    assert!(top > Decimal::ZERO);
}

#[test]
fn sort_records_by_maturity_text() {
    let records = screener_records();
    let bonds: Vec<ScreenerRecord> = by_class(&records, ScreenerAssetType::Bonds)
        .into_iter()
        .cloned()
        .collect();
    let sort = SortState {
        column_id: "maturity_date".into(),
        direction: SortDirection::Asc,
    };
    let sorted = sort_records(bonds.clone(), Some(&sort));
    let maturity_of = |r: &ScreenerRecord| match r {
        ScreenerRecord::Bond(b) => b.maturity_date.clone(),
        _ => String::new(),
    };
    for window in sorted.windows(2) {
        assert!(
            maturity_of(&window[0]) <= maturity_of(&window[1]),
            "maturities not ascending: {} vs {}",
            maturity_of(&window[0]),
            maturity_of(&window[1])
        );
    }
}

#[test]
fn sort_value_record_matches_stock_pipeline() {
    let universe = screener_universe();
    let inst = universe.first().expect("stock universe non-empty");
    let record = ScreenerRecord::Stock(inst.clone());
    let a = match sort_value_record(&record, "price") {
        honba_kao_app_nsh::features::screener::table_grid::SortValue::Number(d) => d,
        honba_kao_app_nsh::features::screener::table_grid::SortValue::Text(_) => {
            panic!("price is a numeric key")
        }
    };
    assert_eq!(a, inst.price);
}

#[test]
fn record_columns_per_class() {
    for (asset, expected_len) in [
        (ScreenerAssetType::Etf, 9),
        (ScreenerAssetType::Bonds, 8),
        (ScreenerAssetType::MutualFunds, 10),
    ] {
        let columns = asset_columns(asset);
        let ids: Vec<String> = columns.iter().map(|c| c.id.to_string()).collect();
        assert_eq!(ids.first().map(String::as_str), Some("symbol"));
        assert_eq!(
            default_visible_columns_for(asset, tab_columns_default()),
            ids,
            "default visible ids equal the class column set ({asset:?})"
        );
        assert_eq!(columns.len(), expected_len);
    }
}

fn tab_columns_default() -> honba_kao_app_nsh::features::screener::types::ScreenerViewTab {
    // Non-stock classes ignore the view tab: any value yields the class set.
    honba_kao_app_nsh::features::screener::types::ScreenerViewTab::default()
}

#[test]
fn default_visible_columns_stock_path_unchanged() {
    for tab in honba_kao_app_nsh::features::screener::types::ScreenerViewTab::ALL {
        let expected: Vec<String> = tab_columns(*tab).iter().map(|c| c.id.to_string()).collect();
        assert_eq!(default_visible_columns(*tab), expected, "tab {tab:?}");
        assert_eq!(
            default_visible_columns_for(ScreenerAssetType::Stocks, *tab),
            expected,
            "stock class keeps the tab-driven defaults"
        );
    }
}

#[test]
fn export_records_csv_sections() {
    let records = screener_records();
    let csv = export_records_csv(&records);
    let lines: Vec<&str> = csv.lines().collect();

    // The stock section emits 1 header + 30 rows = 31 lines; each
    // non-stock class emits 1 header + 10 rows = 11 lines → 64 total.
    assert_eq!(
        lines.len(),
        64,
        "64 lines (31 stock + 3 × (header + 10 rows))"
    );

    let etf_only: Vec<ScreenerRecord> = by_class(&records, ScreenerAssetType::Etf)
        .into_iter()
        .cloned()
        .collect();
    let etf_csv = export_records_csv(&etf_only);
    let mut etf_lines = etf_csv.lines();
    assert_eq!(
        etf_lines.next(),
        Some(
            "SYMBOL,NAME,BRAND,FOCUS,NAV,CHANGE,CHANGE %,AUM (CR),EXPENSE %,TRACKING %,VOLUME,1Y %,3Y %"
        ),
        "etf header exact"
    );
    assert_eq!(etf_lines.count(), 10);

    let bond_only: Vec<ScreenerRecord> = by_class(&records, ScreenerAssetType::Bonds)
        .into_iter()
        .cloned()
        .collect();
    let bond_csv = export_records_csv(&bond_only);
    assert!(bond_csv.starts_with("SYMBOL,NAME,ISSUER,RATING,COUPON %,YTW %,CLEAN PRICE,FACE VALUE,MATURITY,DURATION (YRS),CHANGE %"));
}

#[test]
fn stock_filter_pipeline_untouched() {
    // The host stock tests keep passing (guard: the record layer is additive).
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
fn tile_cap_weight_bins() {
    assert!(
        (tile_size_pts(dec!(500_000)) - 140.0).abs() < f32::EPSILON,
        "mega caps take the wide tile"
    );
    assert!(
        (tile_size_pts(dec!(75_000)) - 100.0).abs() < f32::EPSILON,
        "large caps take the middle tile"
    );
    assert!(
        (tile_size_pts(dec!(5_000)) - 75.0).abs() < f32::EPSILON,
        "mid/small caps take the narrow tile"
    );
}

#[test]
fn view_mode_enum_shape() {
    assert_eq!(ScreenerViewMode::ALL.len(), 2);
    assert_eq!(ScreenerViewMode::default(), ScreenerViewMode::Table);
    assert_eq!(ScreenerViewMode::Table.label(), "Table");
    assert_eq!(ScreenerViewMode::Heatmap.label(), "Heatmap");
}

#[test]
fn compact_cr_notation() {
    assert_eq!(compact_cr(dec!(1_920_000)), "19.2L Cr");
    assert_eq!(compact_cr(dec!(45_000)), "45,000 Cr");
}

#[test]
fn column_menu_filter_binding_map() {
    // §5.4: only the criteria-bound column ids can show the blue "Filter"
    // badge; every other column id stays never-active regardless of state.
    use honba_kao_app_nsh::features::screener::column_header_menu::column_filters_active;

    let mut criteria = FilterCriteria::default();
    assert!(!column_filters_active("price", &criteria));
    assert!(!column_filters_active("pe_ratio", &criteria));
    assert!(!column_filters_active("sector", &criteria));

    criteria.min_price = Some(dec!(100));
    assert!(column_filters_active("price", &criteria));
    criteria.min_price = None;
    criteria.max_price = Some(dec!(200));
    assert!(column_filters_active("price", &criteria));
    criteria.max_price = None;

    criteria.min_pe = Some(dec!(40));
    assert!(column_filters_active("pe_ratio", &criteria));
    criteria.min_pe = None;

    criteria.min_volume = Some(5_000_000);
    assert!(column_filters_active("volume", &criteria));
    criteria.min_volume = None;

    criteria.sector = "Information Technology".to_string();
    assert!(column_filters_active("sector", &criteria));
    criteria.sector = "All".to_string();
    assert!(!column_filters_active("sector", &criteria));

    // A non-bound column stays inactive even with live criteria elsewhere.
    criteria.min_fcf_yield = Some(dec!(5));
    assert!(!column_filters_active("free_cf_cr", &criteria));
}

#[test]
fn column_menu_section_eligibility() {
    // §5.4: the badge starts inactive for every bound column on default
    // criteria — min/max, min-only and max-only bindings are distinct ids.
    use honba_kao_app_nsh::features::screener::column_header_menu::column_filters_active;

    let default_criteria = FilterCriteria::default();
    for col_id in [
        "price",
        "change_percent",
        "pe_ratio",
        "rsi_14",
        "volume",
        "dividend_yield",
        "roe",
        "pb_ratio",
        "sector",
    ] {
        assert!(
            !column_filters_active(col_id, &default_criteria),
            "{col_id} starts unfiltered"
        );
    }
}

#[test]
fn column_menu_badge_never_fires_for_unknown_ids() {
    use honba_kao_app_nsh::features::screener::column_header_menu::column_filters_active;

    let criteria = FilterCriteria {
        max_pb: Some(dec!(3)),
        ..Default::default()
    };
    // ETF/Bond/MF-derived or unknown ids are never criteria-bound.
    assert!(!column_filters_active("aum_cr", &criteria));
    assert!(!column_filters_active("duration_years", &criteria));
    assert!(!column_filters_active("", &criteria));
}
