use honba_kao_app::features::screener::fixtures;
use honba_kao_app::features::screener::heatmap::{
    cap_share_pct, group_sectors, heat_hex, heat_rgb,
};
use rust_decimal_macros::dec;

fn row(
    sector: &str,
    symbol: &str,
    cap_cr: rust_decimal::Decimal,
    change_pct: rust_decimal::Decimal,
) -> honba_kao_app::features::screener::types::ScreenerEquityRecord {
    fixtures::equity_fixture(
        symbol,
        symbol,
        "NSE",
        sector,
        "Generic",
        "Large Cap",
        false,
        false,
        dec!(100),
        change_pct,
        1000,
        1000,
        dec!(0),
        dec!(0),
        dec!(0),
        cap_cr,
        dec!(99),
        dec!(101),
        dec!(90),
        dec!(110),
        dec!(50),
    )
}

#[test]
fn seed_groups_into_six_sector_clusters() {
    let records = fixtures::seed_fixtures();
    let groups = group_sectors(&records);
    assert_eq!(groups.len(), 6);
    assert_eq!(groups.iter().map(|g| g.members.len()).sum::<usize>(), 12);
    let financials = groups
        .iter()
        .find(|g| g.sector == "Financial Services")
        .expect("Financial Services group");
    assert_eq!(financials.members.len(), 5);
    let mut seen = std::collections::HashSet::new();
    for g in &groups {
        for m in &g.members {
            assert!(
                seen.insert(m.symbol.clone()),
                "duplicate {} in heatmap",
                m.symbol
            );
        }
    }
    assert_eq!(seen.len(), 12);
    // Groups render biggest first: total cap strictly non-increasing.
    for pair in groups.windows(2) {
        assert!(pair[0].total_cap_cr >= pair[1].total_cap_cr);
    }
}

#[test]
fn group_totals_average_and_cap_ordering_exact() {
    let records = vec![
        row("Banking", "BBB", dec!(1000), dec!(2.00)),
        row("Banking", "AAA", dec!(2000), dec!(-1.00)),
        row("Energy", "EEE", dec!(3000), dec!(0.50)),
    ];
    let groups = group_sectors(&records);
    assert_eq!(groups.len(), 2);
    // Equal totals (3000) -> alphabetical tiebreak: Banking before Energy.
    assert_eq!(groups[0].sector, "Banking");
    assert_eq!(groups[1].sector, "Energy");
    // Bigger tile first within a sector.
    assert_eq!(groups[0].members[0].symbol, "AAA");
    assert_eq!(groups[0].members[1].symbol, "BBB");
    assert_eq!(groups[0].total_cap_cr, dec!(3000));
    assert_eq!(groups[0].avg_change_pct, dec!(0.50));
    assert_eq!(groups[1].total_cap_cr, dec!(3000));
    assert_eq!(groups[1].avg_change_pct, dec!(0.50));
}

#[test]
fn group_symbol_tiebreak_on_equal_cap() {
    let records = vec![
        row("Tech", "BBB", dec!(100), dec!(1.00)),
        row("Tech", "AAA", dec!(100), dec!(1.00)),
    ];
    let groups = group_sectors(&records);
    assert_eq!(groups[0].members[0].symbol, "AAA");
    assert_eq!(groups[0].members[1].symbol, "BBB");
}

#[test]
fn heat_rgb_matches_documented_anchors_and_clamps() {
    // Documented scale anchors from the screener blueprint.
    assert_eq!(heat_rgb(dec!(-10)), (225, 29, 72)); // clamp low  #e11d48
    assert_eq!(heat_rgb(dec!(-3)), (225, 29, 72)); // #e11d48
    assert_eq!(heat_rgb(dec!(-2)), (244, 63, 94)); // #f43f5e
    assert_eq!(heat_rgb(dec!(-0.50)), (69, 10, 10)); // #450a0a
    assert_eq!(heat_rgb(dec!(0)), (30, 41, 59)); // #1e293b
    assert_eq!(heat_rgb(dec!(0.50)), (6, 78, 59)); // #064e3b
    assert_eq!(heat_rgb(dec!(2)), (16, 185, 129)); // #10b981
    assert_eq!(heat_rgb(dec!(4)), (5, 150, 105)); // #059669
    assert_eq!(heat_rgb(dec!(10)), (5, 150, 105)); // clamp high #059669
}

#[test]
fn heat_rgb_interpolates_piecewise_exactly() {
    // t = (1.00 - 0.50) / (2.00 - 0.50) = 1/3, quantized 4dp.
    assert_eq!(heat_rgb(dec!(1.00)), (9, 114, 82));
    // t = 0.5 midpoint of #450a0a -> #1e293b, half rounding away from zero.
    assert_eq!(heat_rgb(dec!(-0.25)), (50, 26, 35));
}

#[test]
fn heat_hex_formats_lowercase() {
    assert_eq!(heat_hex(dec!(0)), "#1e293b");
    assert_eq!(heat_hex(dec!(2)), "#10b981");
    assert_eq!(heat_hex(dec!(-3)), "#e11d48");
    assert_eq!(heat_hex(dec!(4)), "#059669");
}

#[test]
fn cap_share_pct_proportions() {
    assert_eq!(cap_share_pct(dec!(2000), dec!(3000)), dec!(66.67));
    assert_eq!(cap_share_pct(dec!(1000), dec!(3000)), dec!(33.33));
    assert_eq!(cap_share_pct(dec!(500), dec!(500)), dec!(100.00));
    assert_eq!(cap_share_pct(dec!(100), dec!(0)), dec!(1)); // degenerate guard
}
