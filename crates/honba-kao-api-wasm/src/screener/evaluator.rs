use super::ast::{Operator, Predicate};
use std::collections::HashMap;

pub fn evaluate_predicate(
    pred: &Predicate,
    num_fields: &HashMap<String, f64>,
    text_fields: &HashMap<String, String>,
) -> bool {
    match pred {
        Predicate::Comparison { field, op, value } => {
            if let Some(&actual) = num_fields.get(field) {
                match op {
                    Operator::Equal => (actual - value).abs() < 1e-6,
                    Operator::NotEqual => (actual - value).abs() >= 1e-6,
                    Operator::GreaterThan => actual > *value,
                    Operator::GreaterThanOrEqual => actual >= *value,
                    Operator::LessThan => actual < *value,
                    Operator::LessThanOrEqual => actual <= *value,
                    _ => false,
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
