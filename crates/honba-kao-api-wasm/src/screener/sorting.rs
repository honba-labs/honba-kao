use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SortCriterion {
    pub field: String,
    pub order: SortOrder,
}

pub fn sort_indices<F>(len: usize, mut cmp: F) -> Vec<usize>
where
    F: FnMut(usize, usize) -> std::cmp::Ordering,
{
    let mut indices: Vec<usize> = (0..len).collect();
    indices.sort_by(|&a, &b| cmp(a, b));
    indices
}
