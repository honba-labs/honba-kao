//! Unit tests for `honba-kao-ui-core` home primitives.
//!
//! Covers deterministic, pure-logic surfaces of the Mission Control primitives:
//! LiveClockStrip, MarketHeatStrip, SparklineCell, MetricCard, SystemHealthBar, and QuickNavGrid.

use chrono::{DateTime, Utc};
use honba_kao_ui_core::*;

#[test]
fn live_clock_default_zones_and_formatting() {
    let zones = default_clock_zones();
    assert_eq!(zones.len(), 3);
    assert_eq!(zones[0].id, "ist");
    assert_eq!(zones[0].label, "IST");
    assert_eq!(zones[0].time_zone, "Asia/Kolkata");

    assert_eq!(zones[1].id, "est");
    assert_eq!(zones[1].label, "EST");
    assert_eq!(zones[1].time_zone, "America/New_York");

    assert_eq!(zones[2].id, "utc");
    assert_eq!(zones[2].label, "UTC");
    assert_eq!(zones[2].time_zone, "UTC");

    // Pinned date: 2026-01-15T12:00:00Z (matches OpenTerminalUI test)
    let pinned_dt = DateTime::parse_from_rfc3339("2026-01-15T12:00:00.000Z")
        .expect("valid RFC3339")
        .with_timezone(&Utc);

    assert_eq!(format_clock_for_zone(&pinned_dt, "UTC"), "12:00:00");
    // IST is UTC+5:30 -> 17:30:00
    assert_eq!(
        format_clock_for_zone(&pinned_dt, "Asia/Kolkata"),
        "17:30:00"
    );
    // EST is UTC-5 in Jan -> 07:00:00
    assert_eq!(
        format_clock_for_zone(&pinned_dt, "America/New_York"),
        "07:00:00"
    );
}

#[test]
fn market_heat_strip_tones_and_formatting() {
    assert_eq!(tone_for_change(Some(0.42)), MarketTone::Up);
    assert_eq!(tone_for_change(Some(-1.12)), MarketTone::Down);
    assert_eq!(tone_for_change(Some(0.0)), MarketTone::Neutral);
    assert_eq!(tone_for_change(None), MarketTone::Neutral);
    assert_eq!(tone_for_change(Some(f64::NAN)), MarketTone::Neutral);

    assert_eq!(MarketTone::Up.as_str(), "up");
    assert_eq!(MarketTone::Down.as_str(), "down");
    assert_eq!(MarketTone::Neutral.as_str(), "neutral");

    assert_eq!(MarketFlash::Up.as_str(), "up");
    assert_eq!(MarketFlash::Down.as_str(), "down");

    // Number formatting tests
    assert_eq!(format_number_with_commas(22400.5, 2), "22,400.5");
    assert_eq!(format_number_with_commas(14.2, 2), "14.2");
    assert_eq!(format_number_with_commas(1234567.89, 2), "1,234,567.89");
    assert_eq!(format_number_with_commas(100.0, 2), "100");
    assert_eq!(format_number_with_commas(0.0, 2), "0");
    assert_eq!(format_number_with_commas(-42.5, 2), "-42.5");
    assert_eq!(format_number_with_commas(f64::NAN, 2), "--");
    assert_eq!(format_number_with_commas(f64::INFINITY, 2), "--");

    // Item builders
    let item = MarketHeatStripItem::new("nifty", "NIFTY 50", Some(22400.5), Some(0.42))
        .with_flash(MarketFlash::Up)
        .with_change_label("+0.42%");
    assert_eq!(item.id, "nifty");
    assert_eq!(item.label, "NIFTY 50");
    assert_eq!(item.value, Some(22400.5));
    assert_eq!(item.change_pct, Some(0.42));
    assert_eq!(item.flash, Some(MarketFlash::Up));
    assert_eq!(item.change_label.as_deref(), Some("+0.42%"));
}

#[test]
fn sparkline_geometry_generation() {
    // Empty points
    let empty_geom = build_sparkline_geometry(&[], 132.0, 42.0, 4.0);
    assert_eq!(empty_geom.points.len(), 0);
    assert!(empty_geom.line_path.is_empty());
    assert!(empty_geom.area_path.is_empty());

    // Single point creates flat segment
    let single_geom = build_sparkline_geometry(&[100.0], 132.0, 42.0, 4.0);
    assert_eq!(single_geom.points.len(), 2);
    assert_eq!(single_geom.points[0].value, 100.0);
    assert_eq!(single_geom.points[1].value, 100.0);
    assert!(!single_geom.line_path.is_empty());
    assert!(single_geom.area_path.ends_with('Z'));

    // Multi points
    let points = vec![100.0, 104.0, 110.0, 108.0];
    let geom = build_sparkline_geometry(&points, 132.0, 42.0, 4.0);
    assert_eq!(geom.points.len(), 4);
    assert_eq!(geom.min, 100.0);
    assert_eq!(geom.max, 110.0);
    assert!(geom.line_path.starts_with("M4.00"));
    assert!(geom.area_path.ends_with('Z'));

    // Nearest point index calculation
    assert_eq!(get_nearest_point_index(4, 0.0, 0.0, 100.0), 0);
    assert_eq!(get_nearest_point_index(4, 100.0, 0.0, 100.0), 3);
    assert_eq!(get_nearest_point_index(4, 50.0, 0.0, 100.0), 2); // 0.5 * 3 = 1.5 -> rounds to 2
}

#[test]
fn metric_card_primitives() {
    assert_eq!(MetricTone::Accent.as_str(), "accent");
    assert_eq!(MetricTone::Up.as_str(), "up");
    assert_eq!(MetricTone::Down.as_str(), "down");
    assert_eq!(MetricTone::Neutral.as_str(), "neutral");

    let delta = MetricCardDelta::new("+1,47,000 (+6.12%)", MetricTone::Up);
    assert_eq!(delta.label, "+1,47,000 (+6.12%)");
    assert_eq!(delta.tone, MetricTone::Up);

    let detail = MetricCardDetail::new("Holdings", "12").with_tone(MetricTone::Accent);
    assert_eq!(detail.label, "Holdings");
    assert_eq!(detail.value, "12");
    assert_eq!(detail.tone, MetricTone::Accent);
}

#[test]
fn system_health_tones() {
    assert_eq!(SystemHealthTone::Ok.as_str(), "ok");
    assert_eq!(SystemHealthTone::Warning.as_str(), "warning");
    assert_eq!(SystemHealthTone::Critical.as_str(), "critical");
    assert_eq!(SystemHealthTone::Offline.as_str(), "offline");
    assert_eq!(SystemHealthTone::Stale.as_str(), "stale");
    assert_eq!(SystemHealthTone::Info.as_str(), "info");
    assert_eq!(SystemHealthTone::Neutral.as_str(), "neutral");

    let item = SystemHealthItem::new("ws", "WS", "Connected", SystemHealthTone::Ok);
    assert_eq!(item.id, "ws");
    assert_eq!(item.label, "WS");
    assert_eq!(item.value, "Connected");
    assert_eq!(item.tone, SystemHealthTone::Ok);
}

#[test]
fn quick_nav_grid_structures() {
    let item1 = QuickNavItem::new("m1", "Equity", "[M1]").with_description("Market overview");
    let item2 = QuickNavItem::new("fo", "F&O", "[FO]").with_disabled(true);

    assert_eq!(item1.id, "m1");
    assert_eq!(item1.label, "Equity");
    assert_eq!(item1.shortcut, "[M1]");
    assert_eq!(item1.description.as_deref(), Some("Market overview"));
    assert!(!item1.disabled);

    assert_eq!(item2.id, "fo");
    assert!(item2.disabled);

    let section = QuickNavSection::new("markets", "Markets", vec![item1, item2]);
    assert_eq!(section.id, "markets");
    assert_eq!(section.title, "Markets");
    assert_eq!(section.items.len(), 2);
}
