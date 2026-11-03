//! Deterministic financial fixtures for the feature pages (Stage 3
//! Phase A.3 of the gallery-rs porting guide).
//!
//! Until a page's own backend stream lands (the transports live behind
//! the Phase A.1 web boundary), pages render engine-built organisms from
//! these fixtures. Every price/quantity value is built in
//! `rust_decimal::Decimal` (the zero-floats invariant) and converted to
//! the organisms' pre-formatted string props at the module boundary only;
//! the pages themselves never do float math.

use honba_kao_ui_core::organisms::candlestick_chart::CandleDecimal;
use honba_kao_ui_core::organisms::candlestick_chart::ChartPeriod;
use honba_kao_ui_core::organisms::market_tape::TapeEntry;
use honba_kao_ui_core::organisms::order_book::{BookLevel, SpreadInfo};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Strings for the organisms: Decimals are pre-formatted once, here.
pub(crate) fn fmt(d: Decimal) -> String {
    d.round_dp(2).to_string()
}

/// 30_200 -> "30,200": integer thousands separator (no floats).
pub(crate) fn thousands(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "" };
    let mut digits = v.abs().to_string();
    let mut out = String::new();
    while digits.len() > 3 {
        let split = digits.len() - 3;
        out.insert_str(0, &digits[split..]);
        out.insert(0, ',');
        digits.truncate(split);
    }
    out.insert_str(0, &digits);
    format!("{sign}{out}")
}

/// A candle's six-minute walk around `base`: deterministic (no clock, no
/// RNG), so offline reruns and tests stay byte-for-byte comparable.
pub(crate) fn candles(base: Decimal, count: usize) -> Vec<CandleDecimal> {
    let mut out = Vec::with_capacity(count);
    let mut closes: Vec<Decimal> = Vec::with_capacity(count);
    let mut minute = 9u32 * 60 + 15;
    for i in 0..count {
        // Sawtooth drift around base in whole rupees (integer math only).
        let drift = ((i * 37) % 43) as i64 - 21;
        let close = base + Decimal::from(drift);
        let open = closes.last().copied().unwrap_or(close);
        let high = open.max(close) + dec!(3.5);
        let low = open.min(close) - dec!(3.5);
        let volume = Decimal::from(40_000 + ((i * 937) % 7_000) as i64);
        let label = {
            let m = minute % 60;
            let h = minute / 60;
            format!("{h:02}:{m:02}")
        };
        minute += 6;
        closes.push(close);
        // MA5 over the last five closes at this candle's close.
        let window = &closes[closes.len().saturating_sub(5)..];
        let ma5 = window.iter().copied().reduce(|a, b| a + b).unwrap_or(close)
            / Decimal::from(window.len() as u64);
        out.push(CandleDecimal {
            label,
            open: fmt(open),
            high: fmt(high),
            low: fmt(low),
            close: fmt(close),
            volume: fmt(volume),
            average: fmt(close),
            ma5: fmt(ma5),
            ma10: fmt(ma5),
            ma20: fmt(ma5),
        });
    }
    out
}

/// A 2,60-rupee-wide L2 ladder: ten bids and ten asks with integer sizes.
pub(crate) fn book(base: Decimal) -> (Vec<BookLevel>, Vec<BookLevel>, Option<SpreadInfo>) {
    let mut bids = Vec::with_capacity(10);
    let mut asks = Vec::with_capacity(10);
    for i in 0..10 {
        let tick = Decimal::from(i as i64);
        bids.push(BookLevel {
            price: fmt(base - tick),
            size: fmt(Decimal::from(1_900 + 240 * (9 - i as i64) + 40)),
            order_count: format!("{}", 40 - (i as i64) * 3),
        });
        asks.push(BookLevel {
            price: fmt(base + tick),
            size: fmt(Decimal::from(2_100 + 260 * i as i64 + 40)),
            order_count: format!("{}", 41 + (i as i64) * 3),
        });
    }
    (
        bids,
        asks,
        Some(SpreadInfo {
            absolute: fmt(dec!(2.0)),
            percentage: "0.0088%".to_string(),
        }),
    )
}

/// 24 deterministic prints across the fixture universe.
pub(crate) fn tape() -> Vec<TapeEntry> {
    const NAMES: [(&str, Decimal); 8] = [
        ("RELIANCE", dec!(2912.45)),
        ("TCS", dec!(4118.20)),
        ("HDFCBANK", dec!(1682.85)),
        ("INFY", dec!(1786.30)),
        ("NIFTY", dec!(22842.75)),
        ("BANKNIFTY", dec!(50218.40)),
        ("SBIN", dec!(829.55)),
        ("ITC", dec!(466.10)),
    ];
    (0..24)
        .map(|i| {
            let (symbol, price) = NAMES[i % NAMES.len()];
            let up = (i * 13) % 5 > 1;
            let change = if up {
                format!("+{}%", fmt(dec!(0.35) * Decimal::from((i * 3) as i64 % 7)))
            } else {
                format!("-{}%", fmt(dec!(0.35) * Decimal::from((i * 3) as i64 % 7)))
            };
            TapeEntry {
                symbol: symbol.to_string(),
                price: fmt(price),
                change,
                up,
            }
        })
        .collect()
}

/// One headline string for the three-column layout's tape strip.
pub(crate) fn tape_strip() -> String {
    tape()
        .iter()
        .take(6)
        .map(|t| format!("{} {} {}", t.symbol, t.price, t.change))
        .collect::<Vec<_>>()
        .join("  |  ")
}

/// The fixture universe's scrips (screener grids, watchlists, MTA ladders).
pub(crate) const TICKERS: &[&str] = &[
    "RELIANCE-EQ",
    "TCS-EQ",
    "HDFCBANK-EQ",
    "INFY-EQ",
    "SBIN-EQ",
    "ITC-EQ",
    "LT-EQ",
    "AXISBANK-EQ",
    "BHARTIARTL-EQ",
    "MARUTI-EQ",
    "TITAN-EQ",
    "BAJFINANCE-EQ",
    "ASIANPAINT-EQ",
    "HINDUNILVR-EQ",
    "WIPRO-EQ",
    "TECHM-EQ",
    "JSWSTEEL-EQ",
    "TATASTEEL-EQ",
    "ADANIENT-EQ",
    "NIFTY50-INDEX",
];

/// The chart candles re-sampled for one [`ChartPeriod`]: the same
/// deterministic walk regenerated in Decimal and merged 3-to-1 for `M15`
/// and 6-to-1 for `D1` (never re-parsed from formatted strings). M15 keeps
/// the group's first source label; `D1` labels are `D1`..`D10`; the 1:1
/// periods keep the full 60-candle series.
pub fn candles_for_period(base: Decimal, period: ChartPeriod) -> Vec<CandleDecimal> {
    struct Source {
        label: String,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
    }

    let merge = match period {
        ChartPeriod::M15 => 3,
        ChartPeriod::D1 => 6,
        _ => 1,
    };

    let mut sources: Vec<Source> = Vec::with_capacity(60);
    let mut minute = 9u32 * 60 + 15;
    let mut closes: Vec<Decimal> = Vec::with_capacity(60);
    for i in 0..60usize {
        let drift = ((i * 37) % 43) as i64 - 21;
        let close = base + Decimal::from(drift);
        let open = closes.last().copied().unwrap_or(close);
        let high = open.max(close) + dec!(3.5);
        let low = open.min(close) - dec!(3.5);
        let volume = Decimal::from(40_000 + ((i * 937) % 7_000) as i64);
        let m = minute % 60;
        let h = minute / 60;
        let label = format!("{h:02}:{m:02}");
        minute += 6;
        closes.push(close);
        sources.push(Source {
            label,
            open,
            high,
            low,
            close,
            volume,
        });
    }

    let mut merged: Vec<Source> = Vec::with_capacity(60 / merge);
    for group in sources.chunks(merge) {
        let source = &group[0];
        let high = group.iter().map(|s| s.high).max().unwrap_or(source.high);
        let low = group.iter().map(|s| s.low).min().unwrap_or(source.low);
        let close = group.last().expect("chunks never yields empty").close;
        let volume = group.iter().map(|s| s.volume).sum::<Decimal>();
        let label = if matches!(period, ChartPeriod::D1) {
            format!("D{}", merged.len() + 1)
        } else {
            source.label.clone()
        };
        merged.push(Source {
            label,
            open: source.open,
            high,
            low,
            close,
            volume,
        });
    }

    let merged_closes: Vec<Decimal> = merged.iter().map(|c| c.close).collect();
    merged
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let window = &merged_closes[i.saturating_sub(4)..=i];
            let ma5 = window
                .iter()
                .copied()
                .reduce(|a, b| a + b)
                .unwrap_or(c.close)
                / Decimal::from(window.len() as u64);
            CandleDecimal {
                label: c.label.clone(),
                open: fmt(c.open),
                high: fmt(c.high),
                low: fmt(c.low),
                close: fmt(c.close),
                volume: fmt(c.volume),
                average: fmt(c.close),
                ma5: fmt(ma5),
                ma10: fmt(ma5),
                ma20: fmt(ma5),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Screener universe: 30 canonical NIFTY-50-style equities. All financial
// values are `rust_decimal::Decimal` built from integer cents/hundredths
// (e2 = value × 100); the only f64 in the record is the sparkline's canvas
// feed, converted from Decimal closes at this boundary.
// ---------------------------------------------------------------------------

use crate::features::screener::types::{ScreenerInstrument, TechnicalRating};
use num_traits::ToPrimitive as _;

/// One fixture row, expanded into a full screener instrument. Deterministic:
/// no clock, no RNG. `change` = price × pct/100 (Decimal); day low/high walk
/// a fixed spread around the close; the sparkline interpolates `perf_3m`.
/// Nothing here touches floats except the final sparkline conversion.
#[allow(clippy::too_many_arguments)] // a fixture builder's flat field list
fn screener_row(
    symbol: &str,
    name: &str,
    sector: &str,
    price_e2: i64,
    chg_pct_e2: i64,
    volume: i64,
    avg_volume: i64,
    cap_cr: i64,
    pe_e2: Option<i64>,
    pb_e2: Option<i64>,
    roe_e2: Option<i64>,
    roce_e2: Option<i64>,
    net_margin_e2: Option<i64>,
    dividend_e2: Option<i64>,
    rsi_e2: i64,
    sma20_e2: i64,
    sma50_e2: i64,
    sma200_e2: i64,
    high52_e2: i64,
    low52_e2: i64,
    perf_w_e2: i64,
    perf_m_e2: i64,
    perf_q_e2: i64,
    perf_y_e2: i64,
    perf_ytd_e2: i64,
    rating: &str,
) -> ScreenerInstrument {
    const E2: Decimal = dec!(0.01);
    let price = Decimal::new(price_e2, 2);
    let change_percent = Decimal::new(chg_pct_e2, 2);
    let change = price * change_percent / dec!(100);
    let spread = change.abs() + dec!(1.25);
    let day_high = price + spread * dec!(1.8);
    let day_low = price - spread;
    let _ = E2; // notation marker; all literals below build via Decimal::new

    // Deterministic statement metrics (design doc §5.5) derived from the
    // row's own inputs — no extra call-site arguments, no floats, no
    // clock. Shares are `cap ₹Cr / price ₹` (crore shares), so ₹Cr /
    // crore shares lands back in ₹ per share.
    let dec_i = |v: i64| Decimal::new(v, 2);
    let eps = pe_e2.map(|pe| (price / dec_i(pe)).round_dp(2));
    let payout_ratio = match (dividend_e2, roe_e2) {
        (Some(d), Some(roe)) if roe > 0 => Some((dec_i(d) * dec!(100) / dec_i(roe)).round_dp(1)),
        _ => None,
    };
    let dividend_per_share = dividend_e2.map(|d| (price * dec_i(d) / dec!(100)).round_dp(2));
    let dps_growth_5y = dividend_e2.map(|d| (dec_i(d) * dec!(1.2)).round_dp(1));
    let pat_cr = pe_e2.map(|pe| (Decimal::new(cap_cr, 0) / dec_i(pe)).round_dp(0));
    let revenue_cr = match (pat_cr, net_margin_e2) {
        (Some(pat), Some(m)) if m > 0 => Some((pat * dec!(100) / dec_i(m)).round_dp(0)),
        _ => None,
    };
    let revenue_growth_yoy = Some((dec_i(perf_y_e2) * dec!(0.6)).round_dp(1));
    let ebitda_cr = revenue_cr.map(|rev| (rev * dec!(1.28)).round_dp(0));
    let total_debt_cr = pe_e2.map(|_| (Decimal::new(cap_cr, 0) * dec!(0.30)).round_dp(0));
    let debt_to_equity = total_debt_cr.and_then(|debt| {
        let equity = Decimal::new(cap_cr, 0) - debt;
        (equity > Decimal::ZERO).then(|| (debt / equity).round_dp(2))
    });
    let current_ratio =
        net_margin_e2.map(|m| (dec!(1.0) + dec_i(m).abs() * dec!(0.02)).round_dp(2));
    let book_value_per_share = total_debt_cr.map(|debt| {
        let shares = Decimal::new(cap_cr, 0) / price;
        ((Decimal::new(cap_cr, 0) - debt) / shares).round_dp(2)
    });
    let operating_cf_cr = pat_cr.map(|pat| (pat * dec!(1.6)).round_dp(0));
    let free_cf_cr = operating_cf_cr.map(|cf| (cf * dec!(0.75)).round_dp(0));
    let fcf_yield_pct =
        free_cf_cr.map(|fcf| (fcf * dec!(100) / Decimal::new(cap_cr, 0)).round_dp(2));
    let cash_per_share = book_value_per_share.map(|bv| (bv * dec!(0.18)).round_dp(2));
    let sales_per_share = revenue_cr.map(|rev| {
        let shares = Decimal::new(cap_cr, 0) / price;
        (rev / shares).round_dp(2)
    });

    ScreenerInstrument {
        symbol: symbol.to_string(),
        name: name.to_string(),
        sector: sector.to_string(),
        exchange: "NSE".to_string(),
        price,
        change: change.round_dp(2),
        change_percent: change_percent.round_dp(2),
        day_low: day_low.round_dp(2),
        day_high: day_high.round_dp(2),
        volume,
        avg_volume_30d: avg_volume,
        market_cap_cr: Decimal::new(cap_cr, 0),
        pe_ratio: pe_e2.map(|v| Decimal::new(v, 2)),
        pb_ratio: pb_e2.map(|v| Decimal::new(v, 2)),
        roe: roe_e2.map(|v| Decimal::new(v, 2)),
        roce: roce_e2.map(|v| Decimal::new(v, 2)),
        net_margin: net_margin_e2.map(|v| Decimal::new(v, 2)),
        dividend_yield: dividend_e2.map(|v| Decimal::new(v, 2)),
        rsi_14: Decimal::new(rsi_e2, 2),
        sma_20: Decimal::new(sma20_e2, 2),
        sma_50: Decimal::new(sma50_e2, 2),
        sma_200: Decimal::new(sma200_e2, 2),
        high_52: Decimal::new(high52_e2, 2),
        low_52: Decimal::new(low52_e2, 2),
        perf_1w: Decimal::new(perf_w_e2, 2),
        perf_1m: Decimal::new(perf_m_e2, 2),
        perf_3m: Decimal::new(perf_q_e2, 2),
        perf_1y: Decimal::new(perf_y_e2, 2),
        perf_ytd: Decimal::new(perf_ytd_e2, 2),
        technical_rating: TechnicalRating::parse(rating),
        payout_ratio,
        dividend_per_share,
        dps_growth_5y,
        revenue_cr,
        revenue_growth_yoy,
        ebitda_cr,
        pat_cr,
        eps,
        total_debt_cr,
        debt_to_equity,
        current_ratio,
        book_value_per_share,
        operating_cf_cr,
        free_cf_cr,
        fcf_yield_pct,
        cash_per_share,
        sales_per_share,
        sparkline_closes: screener_sparkline(price, Decimal::new(perf_q_e2, 2)),
    }
}

/// 24 scalar closes interpolating from `perf_3m`'s implied anchor price to
/// `price`, Decimal math, f64 only at the conversion.
fn screener_sparkline(price: Decimal, perf_3m: Decimal) -> Vec<f64> {
    let start = price * (dec!(1) - perf_3m / dec!(100));
    (0..24)
        .map(|step| {
            let t = Decimal::from(step) / Decimal::from(23);
            (start + (price - start) * t).to_f64().unwrap_or(0.0)
        })
        .collect()
}

/// The deterministic screener universe: 30 canonical Indian equities across
/// Energy, Banking, IT, Auto, Metals, Pharma, FMCG, Defense.
pub(crate) fn screener_universe() -> Vec<ScreenerInstrument> {
    vec![
        screener_row(
            "RELIANCE",
            "Reliance Industries Limited",
            "Energy",
            284250,
            187,
            4200000,
            3100000,
            1920000,
            Some(2420),
            Some(210),
            Some(980),
            Some(1240),
            Some(820),
            Some(38),
            5460,
            282200,
            279000,
            261000,
            302490,
            222000,
            90,
            240,
            510,
            -320,
            840,
            "Buy",
        ),
        screener_row(
            "TCS",
            "Tata Consultancy Services Limited",
            "IT Services",
            392010,
            -40,
            1800000,
            2200000,
            1410000,
            Some(2850),
            Some(1480),
            Some(2690),
            Some(3350),
            Some(1910),
            Some(162),
            4620,
            394000,
            396600,
            372000,
            459205,
            331115,
            -120,
            -280,
            -640,
            -210,
            560,
            "Sell",
        ),
        screener_row(
            "HDFCBANK",
            "HDFC Bank Limited",
            "Banking",
            168285,
            92,
            8900000,
            7400000,
            1250000,
            Some(1910),
            Some(260),
            Some(1560),
            Some(1740),
            Some(1360),
            Some(95),
            4870,
            167200,
            165400,
            154800,
            190085,
            135010,
            70,
            210,
            480,
            -180,
            920,
            "Buy",
        ),
        screener_row(
            "INFY",
            "Infosys Limited",
            "IT Services",
            148020,
            230,
            3100000,
            2600000,
            620000,
            Some(2240),
            Some(1120),
            Some(2460),
            Some(3030),
            Some(1680),
            Some(275),
            5180,
            146800,
            144200,
            141000,
            170085,
            124020,
            160,
            390,
            720,
            -450,
            1120,
            "Buy",
        ),
        screener_row(
            "SBIN",
            "State Bank of India",
            "Banking",
            78050,
            -110,
            12400000,
            9800000,
            700000,
            Some(1180),
            Some(190),
            Some(1720),
            Some(940),
            Some(1240),
            Some(185),
            5840,
            79200,
            77500,
            70200,
            90525,
            61540,
            50,
            140,
            -260,
            -680,
            310,
            "Neutral",
        ),
        screener_row(
            "ITC",
            "ITC Limited",
            "FMCG",
            46610,
            45,
            18700000,
            14200000,
            580000,
            Some(2690),
            Some(720),
            Some(2840),
            Some(3410),
            Some(2210),
            Some(322),
            5720,
            46500,
            45800,
            42800,
            55120,
            39860,
            30,
            110,
            380,
            890,
            640,
            "Buy",
        ),
        screener_row(
            "LT",
            "Larsen & Toubro Limited",
            "Industrials",
            361240,
            58,
            2100000,
            1600000,
            495000,
            Some(3310),
            Some(540),
            Some(1680),
            Some(1520),
            Some(790),
            Some(98),
            5210,
            359800,
            356600,
            335200,
            411000,
            300010,
            40,
            170,
            420,
            1050,
            590,
            "Neutral",
        ),
        screener_row(
            "AXISBANK",
            "Axis Bank Limited",
            "Banking",
            112275,
            129,
            7600000,
            6100000,
            345000,
            Some(1240),
            Some(170),
            Some(1610),
            Some(1080),
            Some(1420),
            Some(92),
            5090,
            111400,
            109000,
            102200,
            125240,
            89030,
            110,
            290,
            680,
            -920,
            240,
            "StrongBuy",
        ),
        screener_row(
            "BHARTIARTL",
            "Bharti Airtel Limited",
            "Telecom",
            152260,
            166,
            5800000,
            4400000,
            895000,
            Some(5820),
            Some(1240),
            Some(1460),
            Some(1730),
            Some(980),
            Some(51),
            5370,
            150500,
            147000,
            133000,
            172095,
            115025,
            150,
            410,
            1240,
            4150,
            3320,
            "StrongBuy",
        ),
        screener_row(
            "MARUTI",
            "Maruti Suzuki India Limited",
            "Auto",
            1298050,
            -93,
            400000,
            300000,
            408000,
            Some(2580),
            Some(420),
            Some(1820),
            Some(2140),
            Some(1040),
            Some(112),
            4780,
            1306000,
            1298800,
            1215500,
            1399990,
            1089020,
            -80,
            -220,
            -590,
            1560,
            480,
            "Sell",
        ),
        screener_row(
            "TITAN",
            "Titan Company Limited",
            "Consumer",
            342290,
            161,
            1400000,
            1100000,
            303000,
            Some(7650),
            Some(1620),
            Some(2120),
            Some(2680),
            Some(810),
            Some(44),
            4940,
            337800,
            332000,
            314500,
            394000,
            264010,
            190,
            440,
            1180,
            1840,
            2750,
            "Buy",
        ),
        screener_row(
            "BAJFINANCE",
            "Bajaj Finance Limited",
            "Financials",
            692075,
            139,
            2900000,
            2300000,
            428000,
            Some(2490),
            Some(460),
            Some(1890),
            Some(1420),
            Some(2120),
            Some(46),
            6010,
            682000,
            674200,
            621000,
            782055,
            599030,
            140,
            320,
            890,
            -1420,
            -290,
            "Neutral",
        ),
        screener_row(
            "ASIANPAINT",
            "Asian Paints Limited",
            "Consumer",
            241230,
            -76,
            900000,
            700000,
            231000,
            Some(4420),
            Some(1010),
            Some(2240),
            Some(2890),
            Some(1480),
            Some(144),
            4410,
            242800,
            244600,
            238800,
            289020,
            201055,
            -140,
            -310,
            -860,
            -1240,
            -520,
            "StrongSell",
        ),
        screener_row(
            "HINDUNILVR",
            "Hindustan Unilever Limited",
            "FMCG",
            228240,
            29,
            1200000,
            900000,
            536000,
            Some(5010),
            Some(1080),
            Some(2180),
            Some(2720),
            Some(1640),
            Some(189),
            4260,
            227800,
            226600,
            221000,
            261075,
            199980,
            20,
            160,
            540,
            -810,
            -340,
            "Neutral",
        ),
        screener_row(
            "WIPRO",
            "Wipro Limited",
            "IT Services",
            24560,
            216,
            14900000,
            11600000,
            135000,
            Some(2120),
            Some(280),
            Some(1120),
            Some(1390),
            Some(920),
            Some(22),
            5530,
            24000,
            23600,
            23100,
            32015,
            20840,
            250,
            590,
            1420,
            960,
            1510,
            "StrongBuy",
        ),
        screener_row(
            "TECHM",
            "Tech Mahindra Limited",
            "IT Services",
            128590,
            -70,
            3400000,
            2700000,
            126000,
            Some(1980),
            Some(310),
            Some(1340),
            Some(1620),
            Some(880),
            Some(311),
            5010,
            129200,
            129600,
            128800,
            195510,
            103225,
            -90,
            -160,
            -480,
            -2240,
            -110,
            "Sell",
        ),
        screener_row(
            "JSWSTEEL",
            "JSW Steel Limited",
            "Metals",
            92215,
            47,
            4200000,
            3300000,
            223000,
            Some(2210),
            Some(180),
            Some(1260),
            Some(1310),
            Some(490),
            Some(124),
            5290,
            91900,
            90800,
            86900,
            112220,
            65140,
            40,
            120,
            290,
            -1050,
            -480,
            "Neutral",
        ),
        screener_row(
            "TATASTEEL",
            "Tata Steel Limited",
            "Metals",
            16135,
            116,
            24200000,
            18900000,
            204000,
            Some(2840),
            Some(160),
            Some(920),
            Some(840),
            Some(280),
            Some(205),
            4820,
            16000,
            15800,
            14800,
            21490,
            10120,
            120,
            330,
            960,
            -1820,
            240,
            "Buy",
        ),
        screener_row(
            "ADANIENT",
            "Adani Enterprises Limited",
            "Conglomerate",
            284060,
            165,
            2700000,
            2100000,
            333000,
            Some(2670),
            Some(390),
            Some(1840),
            Some(1590),
            Some(970),
            Some(28),
            6170,
            280200,
            274000,
            252000,
            354030,
            220015,
            170,
            490,
            1310,
            2240,
            1680,
            "StrongBuy",
        ),
        screener_row(
            "SUNPHARMA",
            "Sun Pharmaceutical Industries Limited",
            "Pharma",
            181255,
            126,
            2200000,
            1700000,
            435000,
            Some(3620),
            Some(590),
            Some(1960),
            Some(2340),
            Some(1240),
            Some(71),
            5580,
            179600,
            176600,
            164000,
            198540,
            139025,
            130,
            280,
            620,
            1490,
            970,
            "Buy",
        ),
        screener_row(
            "M&M",
            "Mahindra & Mahindra Limited",
            "Auto",
            310225,
            -135,
            2600000,
            2000000,
            383000,
            Some(2790),
            Some(520),
            Some(2210),
            Some(2590),
            Some(1030),
            Some(68),
            5240,
            314200,
            310000,
            293000,
            346890,
            249040,
            -60,
            -240,
            -790,
            2850,
            610,
            "Sell",
        ),
        screener_row(
            "NTPC",
            "NTPC Limited",
            "Power",
            40285,
            85,
            11400000,
            8900000,
            391000,
            Some(1420),
            Some(180),
            Some(1290),
            Some(1010),
            Some(1060),
            Some(231),
            5760,
            40000,
            39500,
            36000,
            47830,
            25860,
            80,
            220,
            670,
            240,
            590,
            "Neutral",
        ),
        screener_row(
            "POWERGRID",
            "Power Grid Corporation of India Limited",
            "Power",
            34590,
            60,
            8800000,
            7100000,
            327000,
            Some(1540),
            Some(220),
            Some(1260),
            Some(1040),
            Some(910),
            Some(306),
            6020,
            34400,
            34100,
            32200,
            39085,
            23415,
            60,
            190,
            440,
            1420,
            810,
            "Neutral",
        ),
        screener_row(
            "ULTRACEMCO",
            "UltraTech Cement Limited",
            "Cement",
            1128040,
            79,
            300000,
            200000,
            340000,
            Some(4860),
            Some(480),
            Some(1240),
            Some(1520),
            Some(1120),
            Some(62),
            4570,
            1120000,
            1112000,
            1048000,
            1229000,
            810030,
            90,
            260,
            780,
            -630,
            -140,
            "Neutral",
        ),
        screener_row(
            "COALINDIA",
            "Coal India Limited",
            "Energy",
            51275,
            -124,
            7400000,
            5800000,
            315000,
            Some(640),
            Some(110),
            Some(2680),
            Some(3240),
            Some(1980),
            Some(412),
            4490,
            52000,
            52800,
            54400,
            101490,
            16520,
            -130,
            -290,
            -920,
            -1950,
            -2220,
            "StrongSell",
        ),
        screener_row(
            "CIPLA",
            "Cipla Limited",
            "Pharma",
            156230,
            123,
            3100000,
            2400000,
            126000,
            Some(2380),
            Some(340),
            Some(1520),
            Some(1890),
            Some(1740),
            Some(96),
            5810,
            155000,
            153200,
            148000,
            169990,
            119940,
            120,
            360,
            940,
            2210,
            1860,
            "StrongBuy",
        ),
        screener_row(
            "NESTLEIND",
            "Nestle India Limited",
            "FMCG",
            126850,
            -33,
            1600000,
            1300000,
            245000,
            Some(6140),
            Some(1420),
            Some(2380),
            Some(3490),
            Some(1920),
            Some(222),
            3840,
            127200,
            127800,
            126600,
            145230,
            110120,
            -50,
            -120,
            -380,
            -1460,
            -210,
            "Sell",
        ),
        screener_row(
            "TRENT",
            "Trent Limited",
            "Consumer",
            658090,
            174,
            1100000,
            900000,
            234000,
            Some(9820),
            Some(2640),
            Some(3120),
            Some(4180),
            Some(1290),
            Some(18),
            6690,
            646000,
            634000,
            582000,
            790025,
            448055,
            210,
            520,
            1670,
            4480,
            3620,
            "StrongBuy",
        ),
        screener_row(
            "BEL",
            "Bharat Electronics Limited",
            "Defense",
            35240,
            378,
            22400000,
            18100000,
            257000,
            Some(5530),
            Some(1120),
            Some(2740),
            Some(3410),
            Some(2240),
            Some(44),
            4990,
            34600,
            34000,
            32600,
            47015,
            21260,
            310,
            780,
            2420,
            4220,
            5120,
            "StrongBuy",
        ),
        screener_row(
            "HAL",
            "Hindustan Aeronautics Limited",
            "Defense",
            324270,
            283,
            4800000,
            3600000,
            240000,
            Some(2480),
            Some(1610),
            Some(3280),
            Some(4210),
            Some(2410),
            Some(78),
            5180,
            319000,
            313200,
            298000,
            399065,
            141080,
            280,
            690,
            2150,
            5150,
            4790,
            "StrongBuy",
        ),
    ]
}

// ---------------------------------------------------------------------------
// Multi-asset screener fixtures (screener-design-doc.md §4): deterministic
// ETF, bond, and mutual-fund universes — ten rows each, `dec!()` literals
// only (no RNG, no clock, zero floats).
// ---------------------------------------------------------------------------

use crate::features::screener::types::{
    BondCreditRating, BondIssuerType, MfSchemeType, ScreenerBondRecord, ScreenerEtfRecord,
    ScreenerMfRecord,
};

/// Ten deterministic ETF rows: Indian index, sector, and commodity funds.
pub fn screener_etfs() -> Vec<ScreenerEtfRecord> {
    vec![
        ScreenerEtfRecord {
            symbol: "NIFTYBEES".to_string(),
            name: "Nippon India Nifty 50 ETF".to_string(),
            brand: "Nippon".to_string(),
            focus: "Broad Market".to_string(),
            nav: dec!(284.65),
            change: dec!(1.85),
            change_percent: dec!(0.65),
            aum_cr: dec!(38500),
            expense_ratio: dec!(0.05),
            tracking_error: dec!(0.03),
            volume: 8_900_000,
            perf_1y: dec!(14.25),
            perf_3y: dec!(18.40),
        },
        ScreenerEtfRecord {
            symbol: "JUNIORBEES".to_string(),
            name: "Nippon India Nifty Next 50 ETF".to_string(),
            brand: "Nippon".to_string(),
            focus: "Broad Market".to_string(),
            nav: dec!(138.20),
            change: dec!(-1.04),
            change_percent: dec!(-0.75),
            aum_cr: dec!(6200),
            expense_ratio: dec!(0.06),
            tracking_error: dec!(0.08),
            volume: 1_250_000,
            perf_1y: dec!(9.85),
            perf_3y: dec!(22.15),
        },
        ScreenerEtfRecord {
            symbol: "MID150BEES".to_string(),
            name: "Nippon India Nifty Midcap 150 ETF".to_string(),
            brand: "Nippon".to_string(),
            focus: "Mid Cap".to_string(),
            nav: dec!(152.40),
            change: dec!(2.33),
            change_percent: dec!(1.53),
            aum_cr: dec!(3400),
            expense_ratio: dec!(0.07),
            tracking_error: dec!(0.11),
            volume: 860_000,
            perf_1y: dec!(26.75),
            perf_3y: dec!(27.85),
        },
        ScreenerEtfRecord {
            symbol: "BANKBEES".to_string(),
            name: "Nippon India Nifty Bank ETF".to_string(),
            brand: "Nippon".to_string(),
            focus: "Bank Nifty".to_string(),
            nav: dec!(562.85),
            change: dec!(-4.22),
            change_percent: dec!(-0.75),
            aum_cr: dec!(8900),
            expense_ratio: dec!(0.08),
            tracking_error: dec!(0.06),
            volume: 1_480_000,
            perf_1y: dec!(11.35),
            perf_3y: dec!(13.90),
        },
        ScreenerEtfRecord {
            symbol: "ITBEES".to_string(),
            name: "Nippon India Nifty IT ETF".to_string(),
            brand: "Nippon".to_string(),
            focus: "IT".to_string(),
            nav: dec!(42.35),
            change: dec!(0.55),
            change_percent: dec!(1.31),
            aum_cr: dec!(1900),
            expense_ratio: dec!(0.09),
            tracking_error: dec!(0.10),
            volume: 2_750_000,
            perf_1y: dec!(7.25),
            perf_3y: dec!(8.60),
        },
        ScreenerEtfRecord {
            symbol: "PSUBNKBEES".to_string(),
            name: "Nippon India Nifty PSU Bank ETF".to_string(),
            brand: "Nippon".to_string(),
            focus: "PSU Bank".to_string(),
            nav: dec!(118.75),
            change: dec!(1.98),
            change_percent: dec!(1.67),
            aum_cr: dec!(4800),
            expense_ratio: dec!(0.10),
            tracking_error: dec!(0.14),
            volume: 1_920_000,
            perf_1y: dec!(32.40),
            perf_3y: dec!(24.55),
        },
        ScreenerEtfRecord {
            symbol: "GOLDBEES".to_string(),
            name: "Nippon India Gold ETF".to_string(),
            brand: "Nippon".to_string(),
            focus: "Gold".to_string(),
            nav: dec!(82.90),
            change: dec!(0.42),
            change_percent: dec!(0.51),
            aum_cr: dec!(11500),
            expense_ratio: dec!(0.35),
            tracking_error: dec!(0.05),
            volume: 4_600_000,
            perf_1y: dec!(22.85),
            perf_3y: dec!(15.40),
        },
        ScreenerEtfRecord {
            symbol: "SILVERBEES".to_string(),
            name: "Nippon India Silver ETF".to_string(),
            brand: "Nippon".to_string(),
            focus: "Silver".to_string(),
            nav: dec!(96.45),
            change: dec!(-1.09),
            change_percent: dec!(-1.13),
            aum_cr: dec!(2650),
            expense_ratio: dec!(0.35),
            tracking_error: dec!(0.22),
            volume: 3_150_000,
            perf_1y: dec!(18.60),
            perf_3y: dec!(26.30),
        },
        ScreenerEtfRecord {
            symbol: "SETFNIF50".to_string(),
            name: "SBI Nifty 50 ETF".to_string(),
            brand: "SBI".to_string(),
            focus: "Broad Market".to_string(),
            nav: dec!(186.55),
            change: dec!(1.21),
            change_percent: dec!(0.65),
            aum_cr: dec!(16800),
            expense_ratio: dec!(0.05),
            tracking_error: dec!(0.04),
            volume: 3_400_000,
            perf_1y: dec!(14.10),
            perf_3y: dec!(18.25),
        },
        ScreenerEtfRecord {
            symbol: "NIFTYIETF".to_string(),
            name: "ICICI Prudential Nifty 50 ETF".to_string(),
            brand: "ICICI".to_string(),
            focus: "Broad Market".to_string(),
            nav: dec!(291.30),
            change: dec!(1.95),
            change_percent: dec!(0.67),
            aum_cr: dec!(12400),
            expense_ratio: dec!(0.06),
            tracking_error: dec!(0.05),
            volume: 2_900_000,
            perf_1y: dec!(14.35),
            perf_3y: dec!(18.55),
        },
    ]
}

/// Ten deterministic bond rows: sovereign GOIs plus PSU and private paper.
pub fn screener_bonds() -> Vec<ScreenerBondRecord> {
    vec![
        ScreenerBondRecord {
            symbol: "GS7182033".to_string(),
            name: "GOI 7.18% 2033".to_string(),
            issuer_type: BondIssuerType::Sovereign,
            credit_rating: BondCreditRating::Sovereign,
            coupon_rate: dec!(7.18),
            ytw: dec!(7.05),
            clean_price: dec!(101.35),
            face_value: dec!(1000),
            maturity_date: "2033-04-17".to_string(),
            duration_years: dec!(6.4),
            change_percent: dec!(-0.12),
        },
        ScreenerBondRecord {
            symbol: "GS6792032".to_string(),
            name: "GOI 6.79% 2032".to_string(),
            issuer_type: BondIssuerType::Sovereign,
            credit_rating: BondCreditRating::Sovereign,
            coupon_rate: dec!(6.79),
            ytw: dec!(6.92),
            clean_price: dec!(98.85),
            face_value: dec!(1000),
            maturity_date: "2032-12-05".to_string(),
            duration_years: dec!(5.6),
            change_percent: dec!(0.08),
        },
        ScreenerBondRecord {
            symbol: "GS7262062".to_string(),
            name: "GOI 7.26% 2062".to_string(),
            issuer_type: BondIssuerType::Sovereign,
            credit_rating: BondCreditRating::Sovereign,
            coupon_rate: dec!(7.26),
            ytw: dec!(7.35),
            clean_price: dec!(96.40),
            face_value: dec!(1000),
            maturity_date: "2062-12-10".to_string(),
            duration_years: dec!(9.5),
            change_percent: dec!(-0.22),
        },
        ScreenerBondRecord {
            symbol: "NHAI6992037".to_string(),
            name: "NHAI 6.99% 2037".to_string(),
            issuer_type: BondIssuerType::CorporatePsu,
            credit_rating: BondCreditRating::Aaa,
            coupon_rate: dec!(6.99),
            ytw: dec!(7.28),
            clean_price: dec!(97.65),
            face_value: dec!(1000),
            maturity_date: "2037-06-28".to_string(),
            duration_years: dec!(8.2),
            change_percent: dec!(0.15),
        },
        ScreenerBondRecord {
            symbol: "REC7532027".to_string(),
            name: "REC 7.53% 2027".to_string(),
            issuer_type: BondIssuerType::CorporatePsu,
            credit_rating: BondCreditRating::Aaa,
            coupon_rate: dec!(7.53),
            ytw: dec!(7.62),
            clean_price: dec!(99.90),
            face_value: dec!(1000),
            maturity_date: "2027-03-30".to_string(),
            duration_years: dec!(1.1),
            change_percent: dec!(0.05),
        },
        ScreenerBondRecord {
            symbol: "PFC7162035".to_string(),
            name: "PFC 7.16% 2035".to_string(),
            issuer_type: BondIssuerType::CorporatePsu,
            credit_rating: BondCreditRating::Aaa,
            coupon_rate: dec!(7.16),
            ytw: dec!(7.45),
            clean_price: dec!(96.85),
            face_value: dec!(1000),
            maturity_date: "2035-01-25".to_string(),
            duration_years: dec!(7.1),
            change_percent: dec!(-0.18),
        },
        ScreenerBondRecord {
            symbol: "NABARD7252031".to_string(),
            name: "NABARD 7.25% 2031".to_string(),
            issuer_type: BondIssuerType::CorporatePsu,
            credit_rating: BondCreditRating::Aaa,
            coupon_rate: dec!(7.25),
            ytw: dec!(7.38),
            clean_price: dec!(99.25),
            face_value: dec!(1000),
            maturity_date: "2031-07-18".to_string(),
            duration_years: dec!(4.4),
            change_percent: dec!(0.10),
        },
        ScreenerBondRecord {
            symbol: "HUDCO7902034".to_string(),
            name: "HUDCO 7.90% 2034".to_string(),
            issuer_type: BondIssuerType::CorporatePsu,
            credit_rating: BondCreditRating::APlus,
            coupon_rate: dec!(7.90),
            ytw: dec!(8.32),
            clean_price: dec!(97.45),
            face_value: dec!(1000),
            maturity_date: "2034-09-12".to_string(),
            duration_years: dec!(6.2),
            change_percent: dec!(-0.25),
        },
        ScreenerBondRecord {
            symbol: "LICHFL7502033".to_string(),
            name: "LIC HFL 7.50% 2033".to_string(),
            issuer_type: BondIssuerType::CorporatePrivate,
            credit_rating: BondCreditRating::AaPlus,
            coupon_rate: dec!(7.50),
            ytw: dec!(7.95),
            clean_price: dec!(98.10),
            face_value: dec!(1000),
            maturity_date: "2033-05-22".to_string(),
            duration_years: dec!(5.4),
            change_percent: dec!(0.18),
        },
        ScreenerBondRecord {
            symbol: "TATACAPA7802032".to_string(),
            name: "Tata Capital 7.80% 2032".to_string(),
            issuer_type: BondIssuerType::CorporatePrivate,
            credit_rating: BondCreditRating::APlus,
            coupon_rate: dec!(7.80),
            ytw: dec!(8.25),
            clean_price: dec!(97.20),
            face_value: dec!(1000),
            maturity_date: "2032-08-14".to_string(),
            duration_years: dec!(5.1),
            change_percent: dec!(-0.30),
        },
    ]
}

/// Ten deterministic mutual-fund rows across AMC, category, and plan.
pub fn screener_mfs() -> Vec<ScreenerMfRecord> {
    vec![
        ScreenerMfRecord {
            symbol: "PPFCAP".to_string(),
            name: "Parag Parikh Flexi Cap Fund".to_string(),
            amc: "Parag Parikh".to_string(),
            category: "Flexi Cap".to_string(),
            scheme_type: MfSchemeType::DirectGrowth,
            nav: dec!(84.65),
            change: dec!(0.42),
            change_percent: dec!(0.50),
            aum_cr: dec!(92000),
            expense_ratio: dec!(0.63),
            sharpe_ratio: dec!(1.85),
            cagr_1y: dec!(22.40),
            cagr_3y: dec!(26.15),
            cagr_5y: dec!(24.80),
            rating_stars: 5,
        },
        ScreenerMfRecord {
            symbol: "QUANTFLEXI".to_string(),
            name: "Quant Flexi Cap Fund".to_string(),
            amc: "Quant".to_string(),
            category: "Flexi Cap".to_string(),
            scheme_type: MfSchemeType::DirectGrowth,
            nav: dec!(246.30),
            change: dec!(-1.20),
            change_percent: dec!(-0.49),
            aum_cr: dec!(6800),
            expense_ratio: dec!(0.71),
            sharpe_ratio: dec!(1.42),
            cagr_1y: dec!(18.75),
            cagr_3y: dec!(24.90),
            cagr_5y: dec!(28.60),
            rating_stars: 4,
        },
        ScreenerMfRecord {
            symbol: "ICICIBLUE".to_string(),
            name: "ICICI Prudential Bluechip Fund".to_string(),
            amc: "ICICI".to_string(),
            category: "Large Cap".to_string(),
            scheme_type: MfSchemeType::DirectGrowth,
            nav: dec!(112.48),
            change: dec!(0.68),
            change_percent: dec!(0.60),
            aum_cr: dec!(64500),
            expense_ratio: dec!(0.87),
            sharpe_ratio: dec!(1.56),
            cagr_1y: dec!(14.20),
            cagr_3y: dec!(19.35),
            cagr_5y: dec!(17.90),
            rating_stars: 4,
        },
        ScreenerMfRecord {
            symbol: "NIPPSMALL".to_string(),
            name: "Nippon India Small Cap Fund".to_string(),
            amc: "Nippon".to_string(),
            category: "Small Cap".to_string(),
            scheme_type: MfSchemeType::DirectGrowth,
            nav: dec!(178.92),
            change: dec!(1.12),
            change_percent: dec!(0.63),
            aum_cr: dec!(61200),
            expense_ratio: dec!(0.68),
            sharpe_ratio: dec!(1.94),
            cagr_1y: dec!(31.60),
            cagr_3y: dec!(35.25),
            cagr_5y: dec!(29.40),
            rating_stars: 5,
        },
        ScreenerMfRecord {
            symbol: "HDFCMID".to_string(),
            name: "HDFC Mid-Cap Opportunities Fund".to_string(),
            amc: "HDFC".to_string(),
            category: "Mid Cap".to_string(),
            scheme_type: MfSchemeType::DirectGrowth,
            nav: dec!(186.75),
            change: dec!(-0.92),
            change_percent: dec!(-0.49),
            aum_cr: dec!(78400),
            expense_ratio: dec!(0.79),
            sharpe_ratio: dec!(1.63),
            cagr_1y: dec!(16.85),
            cagr_3y: dec!(27.40),
            cagr_5y: dec!(23.15),
            rating_stars: 4,
        },
        ScreenerMfRecord {
            symbol: "AXISBLUE".to_string(),
            name: "Axis Bluechip Fund".to_string(),
            amc: "Axis".to_string(),
            category: "Large Cap".to_string(),
            scheme_type: MfSchemeType::DirectGrowth,
            nav: dec!(62.34),
            change: dec!(0.25),
            change_percent: dec!(0.40),
            aum_cr: dec!(24600),
            expense_ratio: dec!(0.56),
            sharpe_ratio: dec!(1.08),
            cagr_1y: dec!(-3.25),
            cagr_3y: dec!(8.95),
            cagr_5y: dec!(11.40),
            rating_stars: 3,
        },
        ScreenerMfRecord {
            symbol: "SBISMLCAP".to_string(),
            name: "SBI Small Cap Fund".to_string(),
            amc: "SBI".to_string(),
            category: "Small Cap".to_string(),
            scheme_type: MfSchemeType::RegularGrowth,
            nav: dec!(58.21),
            change: dec!(-0.38),
            change_percent: dec!(-0.65),
            aum_cr: dec!(19800),
            expense_ratio: dec!(0.68),
            sharpe_ratio: dec!(1.72),
            cagr_1y: dec!(12.45),
            cagr_3y: dec!(28.60),
            cagr_5y: dec!(26.85),
            rating_stars: 4,
        },
        ScreenerMfRecord {
            symbol: "KOTAKEMRG".to_string(),
            name: "Kotak Emerging Equity Fund".to_string(),
            amc: "Kotak".to_string(),
            category: "Mid Cap".to_string(),
            scheme_type: MfSchemeType::Idcw,
            nav: dec!(106.57),
            change: dec!(0.74),
            change_percent: dec!(0.69),
            aum_cr: dec!(48500),
            expense_ratio: dec!(0.42),
            sharpe_ratio: dec!(1.51),
            cagr_1y: dec!(15.30),
            cagr_3y: dec!(24.75),
            cagr_5y: dec!(21.60),
            rating_stars: 4,
        },
        ScreenerMfRecord {
            symbol: "MIRAEELSS".to_string(),
            name: "Mirae Asset ELSS Tax Saver Fund".to_string(),
            amc: "Mirae".to_string(),
            category: "ELSS".to_string(),
            scheme_type: MfSchemeType::DirectGrowth,
            nav: dec!(46.88),
            change: dec!(0.31),
            change_percent: dec!(0.66),
            aum_cr: dec!(27300),
            expense_ratio: dec!(0.54),
            sharpe_ratio: dec!(1.47),
            cagr_1y: dec!(17.65),
            cagr_3y: dec!(21.80),
            cagr_5y: dec!(19.35),
            rating_stars: 4,
        },
        ScreenerMfRecord {
            symbol: "UTIFLEXI".to_string(),
            name: "UTI Flexi Cap Fund".to_string(),
            amc: "UTI".to_string(),
            category: "Flexi Cap".to_string(),
            scheme_type: MfSchemeType::Idcw,
            nav: dec!(142.36),
            change: dec!(-0.85),
            change_percent: dec!(-0.60),
            aum_cr: dec!(32900),
            expense_ratio: dec!(0.66),
            sharpe_ratio: dec!(1.35),
            cagr_1y: dec!(13.90),
            cagr_3y: dec!(22.45),
            cagr_5y: dec!(18.75),
            rating_stars: 4,
        },
    ]
}
