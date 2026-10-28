use super::ast::{Operator, Predicate};
use rust_decimal::Decimal;
use std::collections::HashMap;

pub fn evaluate_predicate(
    pred: &Predicate,
    num_fields: &HashMap<String, Decimal>,
    text_fields: &HashMap<String, String>,
) -> bool {
    match pred {
        Predicate::Comparison { field, op, value } => {
            if let Some(&actual) = num_fields.get(field) {
                match op {
                    Operator::Equal => actual == *value,
                    Operator::NotEqual => actual != *value,
                    Operator::GreaterThan => actual > *value,
                    Operator::GreaterThanOrEqual => actual >= *value,
                    Operator::LessThan => actual < *value,
                    Operator::LessThanOrEqual => actual <= *value,
                    // `Contains` applies to text fields, not numeric ones.
                    Operator::Contains => false,
                    // #62: snapshot-based crossover semantics. The evaluator
                    // receives a single snapshot, so a crossover is reported
                    // when the field currently sits on the far side of the
                    // threshold (CrossesAbove => actual > value, and vice versa).
                    Operator::CrossesAbove => actual > *value,
                    Operator::CrossesBelow => actual < *value,
                }
            } else {
                false
            }
        }
        Predicate::TextComparison { field, op, value } => {
            if let Some(actual) = text_fields.get(field) {
                match op {
                    Operator::Equal => actual.eq_ignore_ascii_case(value),
                    Operator::NotEqual => !actual.eq_ignore_ascii_case(value),
                    Operator::Contains => actual.to_lowercase().contains(&value.to_lowercase()),
                    _ => false,
                }
            } else {
                false
            }
        }
        Predicate::And(preds) => preds
            .iter()
            .all(|p| evaluate_predicate(p, num_fields, text_fields)),
        Predicate::Or(preds) => preds
            .iter()
            .any(|p| evaluate_predicate(p, num_fields, text_fields)),
        Predicate::Not(p) => !evaluate_predicate(p, num_fields, text_fields),
    }
}
