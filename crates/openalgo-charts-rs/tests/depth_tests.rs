use openalgo_charts::depth::*;

fn lvl(price: f64, size: f64, count: u32) -> DepthLevel {
    DepthLevel { price, size, cumulative_size: 0.0, order_count: count }
}

#[test]
fn cumulative_and_top_of_book() {
    let ladder = DepthLadder::new()
        .with_bids(vec![lvl(99.0, 10.0, 1), lvl(98.0, 20.0, 2), lvl(97.0, 30.0, 3)])
        .with_asks(vec![lvl(101.0, 5.0, 1), lvl(102.0, 15.0, 2)])
        .with_mid(100.0)
        .compute_cumulative();
    assert_eq!(ladder.bids[0].cumulative_size, 10.0);
    assert_eq!(ladder.bids[1].cumulative_size, 30.0);
    assert_eq!(ladder.bids[2].cumulative_size, 60.0);
    assert_eq!(ladder.asks[0].cumulative_size, 5.0);
    assert_eq!(ladder.asks[1].cumulative_size, 20.0);
    assert_eq!(ladder.top_of_book(), (Some(99.0), Some(101.0)));
    assert!((ladder.spread().unwrap() - 2.0).abs() < f64::EPSILON);
    assert!((ladder.mid_price().unwrap() - 100.0).abs() < f64::EPSILON);
}

#[test]
fn mid_price_falls_back_to_top_of_book() {
    let ladder = DepthLadder::new()
        .with_bids(vec![lvl(99.0, 10.0, 1)])
        .with_asks(vec![lvl(101.0, 5.0, 1)])
        .compute_cumulative();
    assert!((ladder.mid_price().unwrap() - 100.0).abs() < f64::EPSILON);
}

#[test]
fn dispatch_hits_best_bid_and_asks() {
    let ladder = DepthLadder::new()
        .with_bids(vec![lvl(99.0, 10.0, 1), lvl(98.0, 20.0, 2)])
        .with_asks(vec![lvl(101.0, 5.0, 1), lvl(102.0, 15.0, 2)])
        .compute_cumulative();
    // Tapping 98.2 must dispatch the level actually clicked (98.0, the nearest
    // bid), not the top of book (99.0). 98.2 is used instead of 98.5 because
    // 98.5 is equidistant from 98.0 and 99.0, so "nearest" is undefined there.
    let hit = ladder.dispatch_at(98.2, true).unwrap();
    assert!(hit.is_bid);
    assert!((hit.price - 98.0).abs() < f64::EPSILON);
    let lift = ladder.dispatch_at(101.5, false).unwrap();
    assert!(!lift.is_bid);
    assert!((lift.price - 101.0).abs() < f64::EPSILON);
    assert!(ladder.dispatch_at(50.0, true).is_some()); // still top of book
    let empty: DepthLadder = DepthLadder::new();
    assert!(empty.dispatch_at(100.0, true).is_none());
}

#[test]
fn depth_series_are_scaled() {
    let ladder = DepthLadder::new()
        .with_bids(vec![lvl(99.5, 10.0, 1)])
        .with_asks(vec![lvl(101.25, 5.0, 1)])
        .compute_cumulative();
    let bids = ladder.bid_depth_series();
    assert_eq!(bids[0].time, 9950); // 99.5 * 100
    assert!((bids[0].value - 10.0).abs() < f64::EPSILON);
    let asks = ladder.ask_depth_series();
    assert_eq!(asks[0].time, 10125);
}
