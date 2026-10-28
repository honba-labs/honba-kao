//! Integration tests for `honba-kao-api-wasm::replay`.
//!
//! Covers the replay clock, fill simulation models, and the replay journal.

use honba_api_wasm::replay::{FillModel, ReplayClock, ReplayEvent, ReplayJournal, simulate_fill_price};

#[test]
fn clock_tick_advances_by_speed() {
    let mut clock = ReplayClock::new(1000, 2.0);
    clock.play();
    clock.step_ms = 1000;
    let after = clock.tick();
    assert_eq!(after, 3000); // 1000 + 1000*2
}

#[test]
fn clock_paused_does_not_advance() {
    let mut clock = ReplayClock::new(1000, 50.0);
    // not playing
    let after = clock.tick();
    assert_eq!(after, 1000);
}

#[test]
fn speed_clamped() {
    let clock = ReplayClock::new(0, 0.0);
    assert!((clock.speed_multiplier - 0.1).abs() < 1e-9);
    let clock2 = ReplayClock::new(0, 100.0);
    assert!((clock2.speed_multiplier - 50.0).abs() < 1e-9);
}

#[test]
fn fill_immediate_uses_order_price() {
    assert!((simulate_fill_price(Some(100.0), 110.0, true, FillModel::Immediate) - 100.0).abs() < 1e-9);
    // no order price -> market
    assert!((simulate_fill_price(None, 110.0, true, FillModel::Immediate) - 110.0).abs() < 1e-9);
}

#[test]
fn fill_next_open_uses_market() {
    assert!((simulate_fill_price(Some(100.0), 110.0, true, FillModel::NextOpen) - 110.0).abs() < 1e-9);
}

#[test]
fn fill_slippage_buy_increases() {
    let px = simulate_fill_price(Some(100.0), 100.0, true, FillModel::SlippageBps(100)); // 1%
    assert!((px - 101.0).abs() < 1e-9);
    let px_sell = simulate_fill_price(Some(100.0), 100.0, false, FillModel::SlippageBps(100));
    assert!((px_sell - 99.0).abs() < 1e-9);
}

#[test]
fn journal_next_event_until_advances_cursor() {
    let events = vec![
        ReplayEvent { timestamp: 100, event_type: "a".into(), payload_json: "{}".into() },
        ReplayEvent { timestamp: 200, event_type: "b".into(), payload_json: "{}".into() },
        ReplayEvent { timestamp: 300, event_type: "c".into(), payload_json: "{}".into() },
    ];
    let mut journal = ReplayJournal::new(events);
    let e1 = journal.next_event_until(150).unwrap();
    assert_eq!(e1.event_type, "a");
    let e2 = journal.next_event_until(250).unwrap();
    assert_eq!(e2.event_type, "b");
    // cursor now at event c (ts 300); asking until 250 returns None
    assert!(journal.next_event_until(250).is_none());
    let e3 = journal.next_event_until(350).unwrap();
    assert_eq!(e3.event_type, "c");
}

#[test]
fn journal_reset() {
    let events = vec![ReplayEvent { timestamp: 100, event_type: "a".into(), payload_json: "{}".into() }];
    let mut journal = ReplayJournal::new(events);
    journal.next_event_until(200);
    journal.reset();
    let e = journal.next_event_until(200).unwrap();
    assert_eq!(e.event_type, "a");
}
