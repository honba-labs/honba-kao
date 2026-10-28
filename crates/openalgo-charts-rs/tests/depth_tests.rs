use openalgo_charts::depth::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn lvl(price: Decimal, size: Decimal, count: u32) -> DepthLevel {
    DepthLevel {
        price,
        size,
        cumulative_size: Decimal::ZERO,
        order_count: count,
    }
}

#[test]
fn cumulative_and_top_of_book() {
    let ladder = DepthLadder::new()
        .with_bids(vec![
            lvl(dec!(99.0), dec!(10.0), 1),
            lvl(dec!(98.0), dec!(20.0), 2),
            lvl(dec!(97.0), dec!(30.0), 3),
        ])
        .with_asks(vec![
            lvl(dec!(101.0), dec!(5.0), 1),
            lvl(dec!(102.0), dec!(15.0), 2),
        ])
        .with_mid(dec!(100.0))
        .compute_cumulative();
    assert_eq!(ladder.bids[0].cumulative_size, dec!(10.0));
    assert_eq!(ladder.bids[1].cumulative_size, dec!(30.0));
    assert_eq!(ladder.bids[2].cumulative_size, dec!(60.0));
    assert_eq!(ladder.asks[0].cumulative_size, dec!(5.0));
    assert_eq!(ladder.asks[1].cumulative_size, dec!(20.0));
    assert_eq!(ladder.top_of_book(), (Some(dec!(99.0)), Some(dec!(101.0))));
    assert_eq!(ladder.spread().unwrap(), dec!(2.0));
    assert_eq!(ladder.mid_price().unwrap(), dec!(100.0));
}

#[test]
fn mid_price_falls_back_to_top_of_book() {
    let ladder = DepthLadder::new()
        .with_bids(vec![lvl(dec!(99.0), dec!(10.0), 1)])
        .with_asks(vec![lvl(dec!(101.0), dec!(5.0), 1)])
        .compute_cumulative();
    assert_eq!(ladder.mid_price().unwrap(), dec!(100.0));
}

#[test]
fn dispatch_hits_best_bid_and_asks() {
    let ladder = DepthLadder::new()
        .with_bids(vec![
            lvl(dec!(99.0), dec!(10.0), 1),
            lvl(dec!(98.0), dec!(20.0), 2),
        ])
        .with_asks(vec![
            lvl(dec!(101.0), dec!(5.0), 1),
            lvl(dec!(102.0), dec!(15.0), 2),
        ])
        .compute_cumulative();
    // Tapping 98.2 must dispatch the level actually clicked (98.0, the nearest
    // bid), not the top of book (99.0). 98.2 is used instead of 98.5 because
    // 98.5 is equidistant from 98.0 and 99.0, so "nearest" is undefined there.
    let hit = ladder.dispatch_at(dec!(98.2), true).unwrap();
    assert!(hit.is_bid);
    assert_eq!(hit.price, dec!(98.0));
    let lift = ladder.dispatch_at(dec!(101.5), false).unwrap();
    assert!(!lift.is_bid);
    assert_eq!(lift.price, dec!(101.0));
    assert!(ladder.dispatch_at(dec!(50.0), true).is_some()); // still top of book
    let empty: DepthLadder = DepthLadder::new();
    assert!(empty.dispatch_at(dec!(100.0), true).is_none());
}

#[test]
fn bids_and_asks_are_sorted_regardless_of_input_order() {
    // Bids given out of order (ascending) and asks given out of order
    // (descending) must still end up sorted best-first.
    let ladder = DepthLadder::new()
        .with_bids(vec![
            lvl(dec!(97.0), dec!(30.0), 3),
            lvl(dec!(99.0), dec!(10.0), 1),
            lvl(dec!(98.0), dec!(20.0), 2),
        ])
        .with_asks(vec![
            lvl(dec!(102.0), dec!(15.0), 2),
            lvl(dec!(101.0), dec!(5.0), 1),
        ])
        .compute_cumulative();
    assert_eq!(
        ladder.bids.iter().map(|l| l.price).collect::<Vec<_>>(),
        vec![dec!(99.0), dec!(98.0), dec!(97.0)]
    );
    assert_eq!(
        ladder.asks.iter().map(|l| l.price).collect::<Vec<_>>(),
        vec![dec!(101.0), dec!(102.0)]
    );
    assert_eq!(ladder.bids[0].cumulative_size, dec!(10.0));
    assert_eq!(ladder.bids[2].cumulative_size, dec!(60.0));
}

#[test]
fn depth_series_are_scaled() {
    let ladder = DepthLadder::new()
        .with_bids(vec![lvl(dec!(99.5), dec!(10.0), 1)])
        .with_asks(vec![lvl(dec!(101.25), dec!(5.0), 1)])
        .compute_cumulative();
    let bids = ladder.bid_depth_series();
    assert_eq!(bids[0].time, 9950); // 99.5 * 100
    assert!((bids[0].value - 10.0).abs() < f64::EPSILON);
    let asks = ladder.ask_depth_series();
    assert_eq!(asks[0].time, 10125);
}
