//! Screener universes — the small, static list of scan universes offered on
//! the Screen Setup Deck (Milestone 2 of `plan-screener.md`, section 2.1).
//! Cross-universe aggregation itself is Milestone 3/4 scope; this module only
//! supplies the catalog of selectable universe ids/labels.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Universe {
    pub id: String,
    pub label: String,
}

/// The static universe catalog (`nse_500`, `all_india`, `us_large_cap`).
pub fn default_universes() -> Vec<Universe> {
    vec![
        Universe {
            id: "nse_500".into(),
            label: "NSE 500".into(),
        },
        Universe {
            id: "all_india".into(),
            label: "All India".into(),
        },
        Universe {
            id: "us_large_cap".into(),
            label: "US Large Cap".into(),
        },
    ]
}
