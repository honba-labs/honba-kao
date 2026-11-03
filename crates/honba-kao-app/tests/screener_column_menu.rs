//! Phase 5: column header menu semantics — pure contracts for the
//! `column_header_menu` module (red-first target for the UI lane).

use honba_kao_app::features::screener::components::column_header_menu::{
    FilterSlot, MoveDirection, filter_slots_for, swap_in_order,
};

#[test]
fn swap_in_order_semantics() {
    let order = vec!["a", "b", "c"];
    // Left = toward the leading edge.
    assert_eq!(
        swap_in_order(&order, "b", MoveDirection::Left).expect("moveable"),
        vec!["b", "a", "c"]
    );
    // Right = toward the trailing edge.
    assert_eq!(
        swap_in_order(&order, "b", MoveDirection::Right).expect("moveable"),
        vec!["a", "c", "b"]
    );
    // Boundary moves are no-ops (None keeps the current order).
    assert_eq!(swap_in_order(&order, "a", MoveDirection::Left), None);
    assert_eq!(swap_in_order(&order, "c", MoveDirection::Right), None);
    // Unknown keys are no-ops too.
    assert_eq!(swap_in_order(&order, "z", MoveDirection::Left), None);
}

#[test]
fn filter_slots_mapping() {
    use FilterSlot::{PeMax, PeMin, PriceMax, PriceMin, RsiMax, RsiMin, RvolMin};
    assert_eq!(
        filter_slots_for("price"),
        Some((Some(PriceMin), Some(PriceMax)))
    );
    assert_eq!(
        filter_slots_for("pe_ratio"),
        Some((Some(PeMin), Some(PeMax)))
    );
    assert_eq!(
        filter_slots_for("forward_pe"),
        Some((Some(PeMin), Some(PeMax)))
    );
    // RVOL has only a min bound in `FilterCriteria`.
    assert_eq!(filter_slots_for("rvol"), Some((Some(RvolMin), None)));
    assert_eq!(
        filter_slots_for("rsi_14"),
        Some((Some(RsiMin), Some(RsiMax)))
    );
    // Every other key is not criteria-bound.
    assert_eq!(filter_slots_for("roe"), None);
    assert_eq!(filter_slots_for("volume"), None);
    assert_eq!(filter_slots_for("market_cap_cr"), None);
}
