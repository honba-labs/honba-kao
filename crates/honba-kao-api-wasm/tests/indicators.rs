//! Integration tests for `honba-kao-api-wasm::indicators`.
//!
//! Covers moving averages (SMA/EMA/VWAP), momentum (RSI/ROC), and volatility
//! (Bollinger Bands / ATR).

use honba_api_wasm::indicators::*;

#[test]
fn sma_warmup_then_values() {
    let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let sma = calculate_sma(&prices, 3);
    assert_eq!(sma.len(), 5);
    assert!(sma[0].is_none());
    assert!(sma[1].is_none());
    assert!((sma[2].unwrap() - 2.0).abs() < 1e-9);
    assert!((sma[3].unwrap() - 3.0).abs() < 1e-9);
}

#[test]
fn sma_period_larger_all_none() {
    assert!(calculate_sma(&[1.0, 2.0], 5).iter().all(|v| v.is_none()));
}

#[test]
fn ema_period_one_equals_price() {
    let prices = vec![10.0, 20.0, 30.0];
    let ema = calculate_ema(&prices, 1);
    assert_eq!(ema.len(), 3);
    assert!((ema[0].unwrap() - 10.0).abs() < 1e-9);
    assert!((ema[1].unwrap() - 20.0).abs() < 1e-9);
}

#[test]
fn ema_period_zero_empty() {
    assert!(calculate_ema(&[1.0, 2.0], 0).is_empty());
}

#[test]
fn vwap_weighted() {
    let prices = vec![10.0, 20.0, 30.0];
    let vols = vec![1.0, 1.0, 2.0];
    let vwap = calculate_vwap(&prices, &vols);
    assert!((vwap[0] - 10.0).abs() < 1e-9);
    assert!((vwap[2] - 22.5).abs() < 1e-9); // (10+20+60)/4 = 22.5
}

#[test]
fn vwap_zero_volume_falls_back() {
    let vwap = calculate_vwap(&[5.0, 6.0], &[0.0, 0.0]);
    assert!((vwap[0] - 5.0).abs() < 1e-9);
    assert!((vwap[1] - 6.0).abs() < 1e-9);
}

#[test]
fn rsi_length_matches_input() {
    let prices = vec![10.0, 11.0, 12.0, 11.5, 12.5, 13.0, 12.0, 11.0];
    let rsi = calculate_rsi(&prices, 3);
    assert_eq!(rsi.len(), prices.len());
}

#[test]
fn rsi_in_bounds() {
    let prices: Vec<f64> = (0..60).map(|i| 100.0 + (i as f64 % 7.0 - 3.0) * 2.0).collect();
    let rsi = calculate_rsi(&prices, 14);
    for v in rsi.iter().flatten() {
        assert!(*v >= 0.0 && *v <= 100.0);
    }
}

#[test]
fn rsi_period_zero_all_none() {
    assert!(calculate_rsi(&[1.0, 2.0], 0).iter().all(|v| v.is_none()));
}

#[test]
fn roc() {
    let prices = vec![100.0, 110.0, 121.0];
    let roc = calculate_roc(&prices, 1);
    assert!(roc[0].is_none());
    assert!((roc[1].unwrap() - 10.0).abs() < 1e-9);
    assert!((roc[2].unwrap() - 10.0).abs() < 1e-9);
}

#[test]
fn roc_period_zero_self_comparison() {
    // period 0 compares each price to itself => roc 0
    let roc = calculate_roc(&[1.0, 2.0, 3.0], 0);
    assert_eq!(roc.len(), 3);
    assert!(roc.iter().all(|v| v.is_some()));
    assert!((roc[0].unwrap() - 0.0).abs() < 1e-9);
}

#[test]
fn bollinger_bands_warmup_and_ordering() {
    let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let bb = calculate_bollinger_bands(&prices, 3, 2.0);
    assert_eq!(bb.upper.len(), 7);
    assert!(bb.upper[0].is_none());
    assert!(bb.upper[2].is_some());
    for i in 0..7 {
        match (bb.upper[i], bb.middle[i], bb.lower[i]) {
            (Some(u), Some(m), Some(l)) => {
                assert!(u >= m);
                assert!(m >= l);
            }
            _ => {}
        }
    }
}

#[test]
fn atr_warmup_and_length() {
    let highs = vec![10.0, 12.0, 14.0, 13.0, 15.0];
    let lows = vec![9.0, 10.0, 11.0, 12.0, 13.0];
    let closes = vec![9.5, 11.0, 13.0, 12.5, 14.0];
    let atr = calculate_atr(&highs, &lows, &closes, 2);
    assert_eq!(atr.len(), 5);
    assert!(atr[0].is_none());
    assert!(atr[1].is_some());
}

#[test]
fn atr_period_zero_all_none() {
    let atr = calculate_atr(&[1.0, 2.0], &[0.0, 1.0], &[0.5, 1.5], 0);
    assert!(atr.iter().all(|v| v.is_none()));
}
