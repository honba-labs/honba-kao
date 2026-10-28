//! Demo data universe. Mirrors `web/src/lib/demo/demoUniverse.ts`: sample
//! records so the UI is fully explorable before the API is wired up.

use crate::shared::types::{
    AccountInfo, Bar, BrokerAccount, BrokerFunds, BrokerStatus, DecisionRecord, Holding,
    MarketQuote, Order, PortfolioSnapshot, PortfolioSummary, Position, StrategySummary,
    SystemStatus, TraderInfo,
};

pub fn demo_traders() -> Vec<TraderInfo> {
    vec![
        TraderInfo {
            trader_id: "trd_nifty01".into(),
            trader_name: "Nifty Momentum".into(),
            ai_model: "deepseek-v4.1".into(),
            exchange: "NSE".into(),
            is_running: true,
            pnl: 14_850.50,
            win_rate: 68.5,
        },
        TraderInfo {
            trader_id: "trd_bank02".into(),
            trader_name: "BankNifty MeanRev".into(),
            ai_model: "claude-sonnet".into(),
            exchange: "NSE".into(),
            is_running: true,
            pnl: -3_200.00,
            win_rate: 52.0,
        },
        TraderInfo {
            trader_id: "trd_it03".into(),
            trader_name: "IT Breakout".into(),
            ai_model: "gpt-5-mini".into(),
            exchange: "BSE".into(),
            is_running: false,
            pnl: 8_420.00,
            win_rate: 61.2,
        },
    ]
}

pub fn demo_positions() -> Vec<Position> {
    vec![
        Position {
            symbol: "RELIANCE.NSE".into(),
            is_buy: true,
            qty: 250,
            avg_price: 2840.50,
            mark_price: 2884.20,
            unrealized_pnl: 10_925.00,
        },
        Position {
            symbol: "HDFCBANK.NSE".into(),
            is_buy: true,
            qty: 500,
            avg_price: 1680.00,
            mark_price: 1692.40,
            unrealized_pnl: 6_200.00,
        },
        Position {
            symbol: "INFY.NSE".into(),
            is_buy: false,
            qty: 300,
            avg_price: 1845.20,
            mark_price: 1852.90,
            unrealized_pnl: -2_310.00,
        },
    ]
}

pub fn demo_orders() -> Vec<Order> {
    vec![
        Order {
            order_id: "ord-101".into(),
            symbol: "TCS.NSE".into(),
            order_type: "LIMIT".into(),
            is_buy: true,
            qty: 100,
            filled_qty: 40,
            price: 4120.00,
            status: "PARTIAL".into(),
        },
        Order {
            order_id: "ord-102".into(),
            symbol: "ICICIBANK.NSE".into(),
            order_type: "TWAP (ALGO)".into(),
            is_buy: true,
            qty: 400,
            filled_qty: 150,
            price: 1255.50,
            status: "RUNNING".into(),
        },
        Order {
            order_id: "ord-103".into(),
            symbol: "TATAMOTORS.NSE".into(),
            order_type: "STOP LOSS".into(),
            is_buy: false,
            qty: 200,
            filled_qty: 0,
            price: 980.00,
            status: "PENDING".into(),
        },
    ]
}

pub fn demo_decisions() -> Vec<DecisionRecord> {
    vec![
        DecisionRecord {
            time: "15:24:02.114".into(),
            trader: "Nifty Momentum".into(),
            symbol: "RELIANCE".into(),
            action: "BUY 50 @ 2884.20".into(),
            approved: true,
            reason: "PASSED (risk limits OK)".into(),
        },
        DecisionRecord {
            time: "15:23:45.082".into(),
            trader: "BankNifty MeanRev".into(),
            symbol: "SBIN".into(),
            action: "SELL 200 @ 812.50".into(),
            approved: false,
            reason: "MAX_DRAWDOWN_LIMIT (breached ₹50,000)".into(),
        },
        DecisionRecord {
            time: "15:21:10.590".into(),
            trader: "IT Breakout".into(),
            symbol: "WIPRO".into(),
            action: "BUY 300 @ 540.00".into(),
            approved: false,
            reason: "CIRCUIT_FILTER (near upper band 5%)".into(),
        },
        DecisionRecord {
            time: "15:18:22.311".into(),
            trader: "Nifty Momentum".into(),
            symbol: "HDFCBANK".into(),
            action: "BUY 100 @ 1690.00".into(),
            approved: true,
            reason: "PASSED (risk limits OK)".into(),
        },
    ]
}

pub fn demo_account() -> AccountInfo {
    AccountInfo {
        total_value: 1_842_650.0,
        available_cash: 620_000.0,
        day_pnl: 14_850.0,
        buying_power: 620_000.0,
    }
}

pub fn demo_status(trader_id: &str) -> SystemStatus {
    SystemStatus {
        trader_id: trader_id.to_string(),
        running: true,
        last_update: "15:24:07".into(),
        cpu_pct: 12.4,
        latency_ms: 14,
    }
}

pub fn demo_quotes() -> Vec<MarketQuote> {
    vec![
        MarketQuote {
            symbol: "RELIANCE".into(),
            name: "Reliance Industries".into(),
            last: 2884.20,
            change_pct: 0.6,
        },
        MarketQuote {
            symbol: "HDFCBANK".into(),
            name: "HDFC Bank".into(),
            last: 1692.40,
            change_pct: -0.3,
        },
        MarketQuote {
            symbol: "INFY".into(),
            name: "Infosys".into(),
            last: 1852.90,
            change_pct: 0.4,
        },
        MarketQuote {
            symbol: "TCS".into(),
            name: "Tata Consultancy".into(),
            last: 4120.00,
            change_pct: 1.1,
        },
        MarketQuote {
            symbol: "SBIN".into(),
            name: "State Bank of India".into(),
            last: 812.50,
            change_pct: -0.8,
        },
    ]
}

pub fn demo_strategies() -> Vec<StrategySummary> {
    vec![
        StrategySummary {
            id: "trend-ema",
            name: "EMA Trend Rider",
            description: "Rides sustained moves with a 20/50 EMA crossover and ATR trailing stop.",
            win_rate: 64.0,
            author: "honba-labs",
        },
        StrategySummary {
            id: "bollinger-squeeze",
            name: "Bollinger Squeeze",
            description: "Fades volatility contractions and plays the expansion break.",
            win_rate: 52.0,
            author: "honba-labs",
        },
        StrategySummary {
            id: "vol-breakout",
            name: "Volume Expansion",
            description: "Enters on relative-volume breakouts above the opening range.",
            win_rate: 58.5,
            author: "community",
        },
    ]
}

/// Simple deterministic pseudo-series used by the chart placeholders.
pub fn demo_price_series(seed: f64, len: usize) -> Vec<f64> {
    let mut out = Vec::with_capacity(len);
    let mut value: f64;
    for i in 0..len {
        let wave = (i as f64 * 0.35).sin() * (seed * 0.004);
        let drift = i as f64 * 0.15;
        value = seed + drift + wave;
        out.push(value);
    }
    out
}

/// Anchored clock for demo charts (2026-10-14T00:00Z) so the rendered series
/// is stable between reloads.
pub const DEMO_EPOCH: i64 = 1_792_000_000;

/// Deterministic OHLCV bars built from [`demo_price_series`].
///
/// `interval_seconds` is the bar width; `count` bars are emitted from
/// [`DEMO_EPOCH`].
pub fn demo_ohlc_bars(seed: f64, count: usize, interval_seconds: i64) -> Vec<Bar> {
    let closes = demo_price_series(seed, count);
    let mut bars = Vec::with_capacity(count);
    let mut previous = seed;

    for (index, close) in closes.into_iter().enumerate() {
        let open = previous;
        let wick = (seed * 0.0015).max(0.5) * (0.4 + (index as f64 * 1.7).sin().abs());
        let high = open.max(close) + wick;
        let low = open.min(close) - wick;
        let volume = 40_000.0 + (index as f64 * 0.9).cos().abs() * 55_000.0;

        bars.push(Bar::new(
            DEMO_EPOCH + interval_seconds * index as i64,
            round2(open),
            round2(high),
            round2(low),
            round2(close),
            volume.round(),
        ));
        previous = close;
    }

    bars
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// ---------------------------------------------------------------------------
// Broker accounts + aggregated portfolio
// ---------------------------------------------------------------------------

/// The six brokers the app can link, in display order. Zerodha and Dhan are
/// seeded as connected so the aggregated view is populated on first load.
pub fn demo_brokers() -> Vec<BrokerAccount> {
    vec![
        BrokerAccount {
            id: "zerodha",
            name: "Zerodha",
            legal_name: "Zerodha Broking Ltd",
            monogram: "Z",
            tone: "--broker-zerodha",
            status: BrokerStatus::Connected,
            client_id: "ZD4821",
            segments: &["Equity", "F&O", "Mutual funds"],
            last_sync: "12s ago",
        },
        BrokerAccount {
            id: "dhan",
            name: "Dhan",
            legal_name: "Dhan Financial Services",
            monogram: "D",
            tone: "--broker-dhan",
            status: BrokerStatus::Connected,
            client_id: "DH1180",
            segments: &["Equity", "F&O"],
            last_sync: "34s ago",
        },
        BrokerAccount {
            id: "upstox",
            name: "Upstox",
            legal_name: "RKSV Securities India",
            monogram: "U",
            tone: "--broker-upstox",
            status: BrokerStatus::Disconnected,
            client_id: "",
            segments: &["Equity", "F&O"],
            last_sync: "—",
        },
        BrokerAccount {
            id: "angel-one",
            name: "Angel One",
            legal_name: "Angel One Ltd",
            monogram: "A",
            tone: "--broker-angel",
            status: BrokerStatus::Disconnected,
            client_id: "",
            segments: &["Equity", "F&O", "Commodity"],
            last_sync: "—",
        },
        BrokerAccount {
            id: "fyers",
            name: "Fyers",
            legal_name: "Fyers Securities",
            monogram: "F",
            tone: "--broker-fyers",
            status: BrokerStatus::Disconnected,
            client_id: "",
            segments: &["Equity", "F&O"],
            last_sync: "—",
        },
        BrokerAccount {
            id: "groww",
            name: "Groww",
            legal_name: "Billionbrains Garage Ventures",
            monogram: "G",
            tone: "--broker-groww",
            status: BrokerStatus::Disconnected,
            client_id: "",
            segments: &["Equity", "Mutual funds"],
            last_sync: "—",
        },
    ]
}

/// Holdings across every broker. The snapshot filters these by connection
/// state, so connecting a broker visibly grows the table.
pub fn demo_holdings() -> Vec<Holding> {
    // One row is inherently eight fields; a builder struct would only add noise.
    #[allow(clippy::too_many_arguments)]
    fn holding(
        symbol: &str,
        name: &str,
        broker: &'static str,
        sector: &'static str,
        qty: i64,
        avg_price: f64,
        last_price: f64,
        day_change_pct: f64,
    ) -> Holding {
        Holding {
            symbol: symbol.to_string(),
            name: name.to_string(),
            broker,
            sector,
            qty,
            avg_price,
            last_price,
            day_change_pct,
        }
    }

    vec![
        holding(
            "RELIANCE",
            "Reliance Industries",
            "zerodha",
            "Energy",
            250,
            2_840.50,
            2_884.20,
            0.82,
        ),
        holding(
            "HDFCBANK",
            "HDFC Bank",
            "zerodha",
            "Financials",
            500,
            1_680.00,
            1_692.40,
            0.41,
        ),
        holding(
            "INFY", "Infosys", "zerodha", "IT", 300, 1_545.00, 1_589.75, -0.36,
        ),
        holding(
            "TCS",
            "Tata Consultancy Services",
            "zerodha",
            "IT",
            120,
            3_890.00,
            3_962.10,
            0.57,
        ),
        holding(
            "TATAMOTORS",
            "Tata Motors",
            "dhan",
            "Auto",
            400,
            985.00,
            1_012.60,
            1.24,
        ),
        holding(
            "BAJFINANCE",
            "Bajaj Finance",
            "dhan",
            "Financials",
            90,
            7_120.00,
            7_045.50,
            -0.72,
        ),
        holding(
            "SBIN",
            "State Bank of India",
            "dhan",
            "Financials",
            600,
            812.00,
            829.35,
            0.66,
        ),
        holding(
            "ICICIBANK",
            "ICICI Bank",
            "upstox",
            "Financials",
            350,
            1_205.00,
            1_228.90,
            0.54,
        ),
        holding(
            "LT",
            "Larsen & Toubro",
            "upstox",
            "Industrials",
            80,
            3_540.00,
            3_612.30,
            0.38,
        ),
        holding(
            "HINDUNILVR",
            "Hindustan Unilever",
            "angel-one",
            "FMCG",
            150,
            2_460.00,
            2_438.15,
            -0.21,
        ),
        holding(
            "MARUTI",
            "Maruti Suzuki",
            "angel-one",
            "Auto",
            40,
            12_640.00,
            12_985.00,
            0.93,
        ),
        holding(
            "ADANIENT",
            "Adani Enterprises",
            "fyers",
            "Conglomerate",
            120,
            3_180.00,
            3_095.40,
            -1.12,
        ),
        holding(
            "ASIANPAINT",
            "Asian Paints",
            "fyers",
            "FMCG",
            200,
            2_890.00,
            2_934.60,
            0.44,
        ),
        holding("ITC", "ITC", "groww", "FMCG", 800, 428.00, 441.25, 0.29),
    ]
}

/// Cash, collateral and margin at each broker.
pub fn demo_funds() -> Vec<BrokerFunds> {
    vec![
        BrokerFunds {
            broker: "zerodha",
            cash: 186_400.0,
            collateral: 62_000.0,
            used_margin: 41_500.0,
        },
        BrokerFunds {
            broker: "dhan",
            cash: 94_250.0,
            collateral: 18_000.0,
            used_margin: 27_600.0,
        },
        BrokerFunds {
            broker: "upstox",
            cash: 58_900.0,
            collateral: 0.0,
            used_margin: 0.0,
        },
        BrokerFunds {
            broker: "angel-one",
            cash: 41_200.0,
            collateral: 0.0,
            used_margin: 0.0,
        },
        BrokerFunds {
            broker: "fyers",
            cash: 36_750.0,
            collateral: 0.0,
            used_margin: 0.0,
        },
        BrokerFunds {
            broker: "groww",
            cash: 22_300.0,
            collateral: 0.0,
            used_margin: 0.0,
        },
    ]
}

/// Full portfolio for the default (seeded) connection state.
pub fn demo_portfolio() -> PortfolioSnapshot {
    build_portfolio(&demo_brokers())
}

/// Recompute the aggregated view for an arbitrary set of broker states.
///
/// Holdings and funds are dropped for disconnected accounts, so this is the
/// single place that defines what "connected" means for the totals.
pub fn build_portfolio(brokers: &[BrokerAccount]) -> PortfolioSnapshot {
    let connected = |id: &str| {
        brokers
            .iter()
            .any(|broker| broker.id == id && broker.status.is_connected())
    };

    let holdings: Vec<Holding> = demo_holdings()
        .into_iter()
        .filter(|holding| connected(holding.broker))
        .collect();
    let funds: Vec<BrokerFunds> = demo_funds()
        .into_iter()
        .filter(|funds| connected(funds.broker))
        .collect();

    let summary = summarize(&holdings, &funds);

    PortfolioSnapshot {
        brokers: brokers.to_vec(),
        holdings,
        funds,
        summary,
    }
}

fn summarize(holdings: &[Holding], funds: &[BrokerFunds]) -> PortfolioSummary {
    let holdings_value: f64 = holdings.iter().map(Holding::value).sum();
    let invested: f64 = holdings.iter().map(Holding::invested).sum();
    let day_pnl: f64 = holdings.iter().map(Holding::day_pnl).sum();
    let unrealised_pnl = holdings_value - invested;

    PortfolioSummary {
        holdings_value,
        invested,
        unrealised_pnl,
        unrealised_pct: if invested == 0.0 {
            0.0
        } else {
            unrealised_pnl / invested * 100.0
        },
        day_pnl,
        cash: funds.iter().map(|funds| funds.cash).sum(),
        collateral: funds.iter().map(|funds| funds.collateral).sum(),
        used_margin: funds.iter().map(|funds| funds.used_margin).sum(),
    }
}
