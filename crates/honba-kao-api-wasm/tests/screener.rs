//! Integration tests for `honba-kao-api-wasm::screener`.
//!
//! Covers the AST predicate evaluator and multi-column sorting.

use honba_api_wasm::screener::ast::{Operator, Predicate};
use honba_api_wasm::screener::{evaluate_predicate, sort_indices, SortCriterion, SortOrder};
use std::collections::HashMap;

#[test]
fn evaluator_numeric_comparison() {
    let mut num = HashMap::new();
    num.insert("price".into(), 150.0);
    num.insert("volume".into(), 1_000_000.0);

    let pred = Predicate::Comparison { field: "price".into(), op: Operator::GreaterThan, value: 100.0 };
    assert!(evaluate_predicate(&pred, &num, &HashMap::new()));

    let pred = Predicate::Comparison { field: "price".into(), op: Operator::LessThan, value: 100.0 };
    assert!(!evaluate_predicate(&pred, &num, &HashMap::new()));

    let pred = Predicate::Comparison { field: "volume".into(), op: Operator::GreaterThanOrEqual, value: 1_000_000.0 };
    assert!(evaluate_predicate(&pred, &num, &HashMap::new()));
}

#[test]
fn evaluator_missing_field_false() {
    let pred = Predicate::Comparison { field: "missing".into(), op: Operator::Equal, value: 1.0 };
    assert!(!evaluate_predicate(&pred, &HashMap::new(), &HashMap::new()));
}

#[test]
fn evaluator_text_contains() {
    let mut text = HashMap::new();
    text.insert("sector".into(), "Technology".into());
    let pred = Predicate::TextComparison { field: "sector".into(), op: Operator::Contains, value: "tech".into() };
    assert!(evaluate_predicate(&pred, &HashMap::new(), &text));
}

#[test]
fn evaluator_and_or_not() {
    let mut num = HashMap::new();
    num.insert("price".into(), 150.0);
    num.insert("volume".into(), 1_000_000.0);

    let and = Predicate::And(vec![
        Predicate::Comparison { field: "price".into(), op: Operator::GreaterThan, value: 100.0 },
        Predicate::Comparison { field: "volume".into(), op: Operator::GreaterThan, value: 500_000.0 },
    ]);
    assert!(evaluate_predicate(&and, &num, &HashMap::new()));

    let or = Predicate::Or(vec![
        Predicate::Comparison { field: "price".into(), op: Operator::LessThan, value: 10.0 },
        Predicate::Comparison { field: "volume".into(), op: Operator::GreaterThan, value: 500_000.0 },
    ]);
    assert!(evaluate_predicate(&or, &num, &HashMap::new()));

    let not = Predicate::Not(Box::new(Predicate::Comparison {
        field: "price".into(),
        op: Operator::Equal,
        value: 150.0,
    }));
    assert!(!evaluate_predicate(&not, &num, &HashMap::new()));
}

#[test]
fn sort_indices_ascending_descending() {
    let values = vec![10.0, 30.0, 20.0];
    let asc = sort_indices(values.len(), |a, b| values[a].partial_cmp(&values[b]).unwrap());
    assert_eq!(asc, vec![0, 2, 1]);
    let desc = sort_indices(values.len(), |a, b| values[b].partial_cmp(&values[a]).unwrap());
    assert_eq!(desc, vec![1, 2, 0]);
}

#[test]
fn sort_criterion_fields() {
    let c = SortCriterion { field: "price".into(), order: SortOrder::Ascending };
    assert_eq!(c.field, "price");
    assert_eq!(c.order, SortOrder::Ascending);
}
