//! Deterministic, zero-float telemetry math for the Futuristic Neural AST
//! Screener (`FunnelTelemetry`, `LaserGauge`, `ConstellationPoint`, `RadarAxes`).
//!
//! All coordinates, percentages, and reward-to-risk ratios are computed in
//! 128-bit `rust_decimal::Decimal`.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use super::filters::filter_records;
use super::indices::{benchmark_for, member_count};
use super::scanner::{SETUP_UNQUALIFIED, scan_row};
use super::types::{FilterCriteria, ScreenerEquityRecord};

/// Live Universe Reduction Funnel counts (`Total -> Index -> Matched -> Qualified`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunnelTelemetry {
    pub total_universe: usize,
    pub index_universe: usize,
    pub matched_count: usize,
    pub qualified_count: usize,
}

/// Compute the 4-stage reduction funnel over `universe` and `criteria`.
pub fn compute_funnel(
    universe: &[ScreenerEquityRecord],
    criteria: &FilterCriteria,
) -> FunnelTelemetry {
    let total_universe = universe.len();
    let index_key = criteria.market_index.as_str();
    let index_universe =
        if !index_key.is_empty() && index_key != "ALL" && benchmark_for(index_key).is_some() {
            member_count(index_key)
        } else {
            total_universe
        };
    let matched = filter_records(universe, criteria);
    let matched_count = matched.len();
    let qualified_count = matched
        .iter()
        .filter(|r| scan_row(r).setup_type != SETUP_UNQUALIFIED)
        .count();

    FunnelTelemetry {
        total_universe,
        index_universe,
        matched_count,
        qualified_count,
    }
}

/// Invalidation-to-Trigger Laser Gauge telemetry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaserGauge {
    /// Clamped 0..=100 marker position along the `[invalidation, trigger]` span.
    pub position_pct: Decimal,
    /// Signed percentage distance from `price` to `invalidation_level`.
    pub dist_to_invalidation_pct: Decimal,
    /// Signed percentage distance from `price` to `trigger_level`.
    pub dist_to_trigger_pct: Decimal,
    /// Reward-to-risk ratio `(trigger - price) / (price - invalidation)`, rounded 2dp.
    pub reward_risk_ratio: Decimal,
}

fn clamp_decimal(val: Decimal, low: Decimal, high: Decimal) -> Decimal {
    if val < low {
        low
    } else if val > high {
        high
    } else {
        val
    }
}

/// Compute the exact Decimal laser gauge metrics for `price` relative to
/// `invalidation` and `trigger`.
pub fn compute_laser_gauge(price: Decimal, invalidation: Decimal, trigger: Decimal) -> LaserGauge {
    let span = trigger - invalidation;
    let raw_pct = if span > Decimal::ZERO {
        ((price - invalidation) * dec!(100) / span).round_dp(2)
    } else {
        dec!(50.00)
    };
    let position_pct = clamp_decimal(raw_pct, Decimal::ZERO, dec!(100));

    let dist_to_invalidation_pct = if price > Decimal::ZERO {
        ((invalidation - price) * dec!(100) / price).round_dp(2)
    } else {
        Decimal::ZERO
    };
    let dist_to_trigger_pct = if price > Decimal::ZERO {
        ((trigger - price) * dec!(100) / price).round_dp(2)
    } else {
        Decimal::ZERO
    };

    let risk = price - invalidation;
    let reward = trigger - price;
    let reward_risk_ratio = if risk > Decimal::ZERO && reward > Decimal::ZERO {
        (reward / risk).round_dp(2)
    } else {
        Decimal::ZERO
    };

    LaserGauge {
        position_pct,
        dist_to_invalidation_pct,
        dist_to_trigger_pct,
        reward_risk_ratio,
    }
}

/// 2D Alpha Constellation node coordinates and metadata (100% `Decimal`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstellationPoint {
    pub symbol: String,
    pub sector: String,
    pub price: Decimal,
    pub change_pct: Decimal,
    pub rsi_14: Decimal,
    pub rvol: Decimal,
    pub setup_type: String,
    pub score: Decimal,
    /// Normalized X coordinate (0..=100) mapped from RSI(14) 25..=85.
    pub x_pct: Decimal,
    /// Normalized Y coordinate (0..=100, top=0) mapped from RVOL 0.5x..=3.5x.
    pub y_pct: Decimal,
    /// Node radius in SVG/CSS pixels (8..=20) scaled by AST scanner score.
    pub radius_px: Decimal,
}

/// Project a slice of `ScreenerEquityRecord`s onto the 2D Alpha Constellation plane.
pub fn compute_constellation(records: &[ScreenerEquityRecord]) -> Vec<ConstellationPoint> {
    records
        .iter()
        .map(|r| {
            let scan = scan_row(r);
            // X maps RSI [25..85] -> [8..92]
            let rsi_clamped = clamp_decimal(r.rsi_14, dec!(25), dec!(85));
            let x_pct = (dec!(8) + (rsi_clamped - dec!(25)) * dec!(84) / dec!(60)).round_dp(2);

            // Y maps RVOL [0.50..3.50] -> [88..12] (higher RVOL is higher up, i.e. smaller Y)
            let rvol_clamped = clamp_decimal(r.rvol, dec!(0.50), dec!(3.50));
            let y_pct =
                (dec!(88) - (rvol_clamped - dec!(0.50)) * dec!(76) / dec!(3.00)).round_dp(2);

            let radius_px = (dec!(8) + (scan.score * dec!(12) / dec!(100))).round_dp(1);

            ConstellationPoint {
                symbol: r.symbol.clone(),
                sector: r.sector.clone(),
                price: r.price,
                change_pct: r.change_pct,
                rsi_14: r.rsi_14,
                rvol: r.rvol,
                setup_type: scan.setup_type,
                score: scan.score,
                x_pct: clamp_decimal(x_pct, Decimal::ZERO, dec!(100)),
                y_pct: clamp_decimal(y_pct, Decimal::ZERO, dec!(100)),
                radius_px: clamp_decimal(radius_px, dec!(8), dec!(22)),
            }
        })
        .collect()
}

/// 5-axis normalized factor scores (0..=100) for the Target Inspector Pentagon Radar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadarAxes {
    pub squeeze: Decimal,
    pub trend: Decimal,
    pub rvol: Decimal,
    pub rsi: Decimal,
    pub roe: Decimal,
}

/// Fixed-point unit vectors `(cos, sin)` for a regular pentagon starting at -90 deg (top):
/// Angles: -90°, -18°, +54°, +126°, +198°.
const PENTAGON_UNIT_VECTORS: [(Decimal, Decimal); 5] = [
    (dec!(0.0000), dec!(-1.0000)),
    (dec!(0.9511), dec!(-0.3090)),
    (dec!(0.5878), dec!(0.8090)),
    (dec!(-0.5878), dec!(0.8090)),
    (dec!(-0.9511), dec!(-0.3090)),
];

impl RadarAxes {
    /// Format the 5 vertices as an SVG `points` string centered at `(80, 75)` with max radius `50`.
    pub fn svg_polygon_points(&self) -> String {
        let cx = dec!(80);
        let cy = dec!(75);
        let max_r = dec!(50);
        let scores = [self.squeeze, self.trend, self.rvol, self.rsi, self.roe];

        scores
            .iter()
            .zip(PENTAGON_UNIT_VECTORS.iter())
            .map(|(score, (ux, uy))| {
                let r = max_r * clamp_decimal(*score, dec!(15), dec!(100)) / dec!(100);
                let x = (cx + r * *ux).round_dp(1);
                let y = (cy + r * *uy).round_dp(1);
                format!("{x},{y}")
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format a reference pentagon ring at `scale_pct` (e.g., `50` or `100`).
    pub fn svg_ring_points(scale_pct: Decimal) -> String {
        let cx = dec!(80);
        let cy = dec!(75);
        let r = dec!(50) * scale_pct / dec!(100);
        PENTAGON_UNIT_VECTORS
            .iter()
            .map(|(ux, uy)| {
                let x = (cx + r * *ux).round_dp(1);
                let y = (cy + r * *uy).round_dp(1);
                format!("{x},{y}")
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Compute the 5 normalized (0..=100) radar axes for `record`.
pub fn compute_radar_axes(record: &ScreenerEquityRecord) -> RadarAxes {
    // 1. Squeeze: narrower bandwidth (< 5%) scores higher; if unset (0), baseline 55.
    let squeeze = if record.bb_bandwidth > Decimal::ZERO {
        let raw = (dec!(10) - record.bb_bandwidth) * dec!(12);
        clamp_decimal(raw.round_dp(1), dec!(15), dec!(100))
    } else {
        dec!(55)
    };

    // 2. Trend: price vs SMA 200 (or 52W range position when SMA 200 is 0).
    let trend = if record.sma_200 > Decimal::ZERO {
        let ratio = record.price * dec!(80) / record.sma_200;
        clamp_decimal(ratio.round_dp(1), dec!(20), dec!(100))
    } else if record.high_52w > record.low_52w {
        let pos = (record.price - record.low_52w) * dec!(100) / (record.high_52w - record.low_52w);
        clamp_decimal(pos.round_dp(1), dec!(20), dec!(100))
    } else {
        dec!(60)
    };

    // 3. RVOL: 3.0x maps to 100
    let rvol = clamp_decimal(
        (record.rvol * dec!(100) / dec!(3)).round_dp(1),
        dec!(15),
        dec!(100),
    );

    // 4. RSI: 0..100 directly clamped
    let rsi = clamp_decimal(record.rsi_14.round_dp(1), dec!(15), dec!(100));

    // 5. ROE: 30% ROE maps to 100
    let roe = clamp_decimal(
        (record.roe * dec!(100) / dec!(30)).round_dp(1),
        dec!(15),
        dec!(100),
    );

    RadarAxes {
        squeeze,
        trend,
        rvol,
        rsi,
        roe,
    }
}
