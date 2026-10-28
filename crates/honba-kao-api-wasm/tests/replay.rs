//! Integration tests for `honba-kao-api-wasm::replay`.
//!
//! Covers the replay clock, fill simulation models, and the replay journal.
//! All monetary math uses `rust_decimal::Decimal` (#54).

use honba_api_wasm::replay::{FillModel, ReplayClock, ReplayEvent, ReplayJournal, simulate_fill_price};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

fn d(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap()
}

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
    assert_eq!(clock.speed_multiplier, 0.1);
    let clock2 = ReplayClock::new(0, 100.0);
    assert_eq!(clock2.speed_multiplier, 50.0);
}

// #61: sub-millisecond fractional remainder keeps the clock advancing at
// slow-motion speeds (step_ms=1, speed=0.5x => advance 0 per tick truncates
// without the remainder).
#[test]
fn clock_slow_motion_advances_via_remainder() {
    let mut clock = ReplayClock::new(0, 0.5);
    clock.play();
    clock.step_ms = 1;
    // two ticks of 1ms * 0.5 = 0.5ms each => remainder carries, total advances 1ms
    clock.tick();
    let after_first = clock.current_timestamp;
    clock.tick();
    assert!(clock.current_timestamp > after_first, "clock must advance at 0.5x speed");
}

#[test]
fn fill_immediate_uses_order_price() {
    assert_eq!(simulate_fill_price(Some(d(100.0)), d(110.0), true, FillModel::Immediate), d(100.0));
    // no order price -> market
    assert_eq!(simulate_fill_price(None, d(110.0), true, FillModel::Immediate), d(110.0));
}

#[test]
fn fill_next_open_uses_market() {
    assert_eq!(simulate_fill_price(Some(d(100.0)), d(110.0), true, FillModel::NextOpen), d(110.0));
}

// #61: a limit buy cannot fill above its limit price; a limit sell cannot fill
// below its limit price (slippage is capped at the limit).
#[test]
fn fill_slippage_capped_at_limit() {
    let px = simulate_fill_price(Some(d(100.0)), d(100.0), true, FillModel::SlippageBps(100)); // 1%
    assert_eq!(px, d(100.0)); // 101 capped at limit 100
    let px_sell = simulate_fill_price(Some(d(100.0)), d(100.0), false, FillModel::SlippageBps(100));
    assert_eq!(px_sell, d(100.0)); // 99 floored at limit 100
    // market order (no limit) still slips
    let mkt = simulate_fill_price(None, d(100.0), true, FillModel::SlippageBps(100));
    assert_eq!(mkt, d(101.0));
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

// #61: out-of-order events are sorted in ReplayJournal::new so the cursor never
// halts prematurely.
#[test]
fn journal_sorts_out_of_order_events() {
    let events = vec![
        ReplayEvent { timestamp: 300, event_type: "c".into(), payload_json: "{}".into() },
        ReplayEvent { timestamp: 100, event_type: "a".into(), payload_json: "{}".into() },
        ReplayEvent { timestamp: 200, event_type: "b".into(), payload_json: "{}".into() },
    ];
    let mut journal = ReplayJournal::new(events);
    assert_eq!(journal.next_event_until(150).unwrap().event_type, "a");
    assert_eq!(journal.next_event_until(250).unwrap().event_type, "b");
    assert_eq!(journal.next_event_until(350).unwrap().event_type, "c");
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
