//! Heatmap math: sector clustering, cap-weighted tile shares, and
//! `change_pct` -> RGB heat interpolation. Pure domain logic (zero floats).

use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::prelude::RoundingStrategy;
use rust_decimal_macros::dec;

use super::types::ScreenerEquityRecord;

/// One sector cluster for heatmap rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorHeatGroup {
    pub sector: String,
    pub total_cap_cr: Decimal,
    pub avg_change_pct: Decimal,
    pub members: Vec<ScreenerEquityRecord>,
}

/// Group `records` by `sector`. Groups are ordered by total cap descending
/// with an ascending lowercase sector-name tiebreak; members inside a group
/// are ordered by cap descending with an ascending lowercase symbol tiebreak.
pub fn group_sectors(records: &[ScreenerEquityRecord]) -> Vec<SectorHeatGroup> {
    let mut sectors: Vec<String> = records.iter().map(|r| r.sector.clone()).collect();
    sectors.sort();
    sectors.dedup();

    let mut groups = Vec::with_capacity(sectors.len());
    for sector in sectors {
        let mut members: Vec<ScreenerEquityRecord> = records
            .iter()
            .filter(|r| r.sector == sector)
            .cloned()
            .collect();
        let total_cap_cr = members.iter().map(|m| m.market_cap_cr).sum();
        let count = Decimal::from(members.len().max(1) as u64);
        let avg_change_pct =
            (members.iter().map(|m| m.change_pct).sum::<Decimal>() / count).round_dp(2);
        members.sort_by(|a, b| {
            b.market_cap_cr
                .cmp(&a.market_cap_cr)
                .then_with(|| a.symbol.to_lowercase().cmp(&b.symbol.to_lowercase()))
        });
        groups.push(SectorHeatGroup {
            sector,
            total_cap_cr,
            avg_change_pct,
            members,
        });
    }

    groups.sort_by(|a, b| {
        b.total_cap_cr
            .cmp(&a.total_cap_cr)
            .then_with(|| a.sector.to_lowercase().cmp(&b.sector.to_lowercase()))
    });
    groups
}

/// Documented heat scale anchors, ascending by `change_pct` position:
/// `#e11d48` (≤ -3%), `#f43f5e` (-2), `#450a0a` (-0.5), `#1e293b` (0),
/// `#064e3b` (+0.5), `#10b981` (+2), `#059669` (≥ +4%).
const HEAT_ANCHORS: [(Decimal, u8, u8, u8); 7] = [
    (dec!(-3.00), 225, 29, 72),
    (dec!(-2.00), 244, 63, 94),
    (dec!(-0.50), 69, 10, 10),
    (dec!(0.00), 30, 41, 59),
    (dec!(0.50), 6, 78, 59),
    (dec!(2.00), 16, 185, 129),
    (dec!(4.00), 5, 150, 105),
];

#[inline]
fn lerp_channel(low: u8, high: u8, t: Decimal) -> u8 {
    let from = Decimal::from(low);
    let to = Decimal::from(high);
    let mid =
        (from + (to - from) * t).round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
    mid.to_u8().unwrap_or(0)
}

/// Piecewise-linear `change_pct` -> (r, g, b) interpolation across the
/// documented scale anchors; clamps beyond the ends to the extreme colors.
pub fn heat_rgb(change_pct: Decimal) -> (u8, u8, u8) {
    let last = HEAT_ANCHORS.len() - 1;
    if change_pct <= HEAT_ANCHORS[0].0 {
        let (_, r, g, b) = HEAT_ANCHORS[0];
        return (r, g, b);
    }
    if change_pct >= HEAT_ANCHORS[last].0 {
        let (_, r, g, b) = HEAT_ANCHORS[last];
        return (r, g, b);
    }
    for window in HEAT_ANCHORS.windows(2) {
        let (low, low_r, low_g, low_b) = window[0];
        let (high, high_r, high_g, high_b) = window[1];
        if change_pct >= low && change_pct <= high {
            let t = ((change_pct - low) / (high - low)).round_dp(4);
            return (
                lerp_channel(low_r, high_r, t),
                lerp_channel(low_g, high_g, t),
                lerp_channel(low_b, high_b, t),
            );
        }
    }
    let (_, r, g, b) = HEAT_ANCHORS[last];
    (r, g, b)
}

/// `heat_rgb` rendered as a lowercase `"#rrggbb"` CSS hex string.
pub fn heat_hex(change_pct: Decimal) -> String {
    let (r, g, b) = heat_rgb(change_pct);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Tile share of its sector cap as a percent (round_dp(2)). If `total_cap`
/// is zero, returns `dec!(1)` (tiny flex-grow placeholder).
pub fn cap_share_pct(cap: Decimal, total_cap: Decimal) -> Decimal {
    if total_cap == Decimal::ZERO {
        return dec!(1);
    }
    (cap * dec!(100) / total_cap).round_dp(2)
}
