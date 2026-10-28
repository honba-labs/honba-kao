use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    CrossesAbove,
    CrossesBelow,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Predicate {
    Comparison {
        field: String,
        op: Operator,
        value: f64,
    },
    TextComparison {
        field: String,
        op: Operator,
        value: String,
    },
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
    Not(Box<Predicate>),
}
