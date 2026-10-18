//! Demo data universe. Mirrors `web/src/lib/demo/demoUniverse.ts`: sample
//! records so the UI is fully explorable before the API is wired up.

use crate::types::{
    AccountInfo, DecisionRecord, MarketQuote, Order, Position, StrategySummary, SystemStatus,
    TraderInfo,
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
