//! Item model for the CommandPalette (`Ctrl+K`).
//!
//! Port of the item assembly in OpenTerminalUI `layout/CommandPalette.tsx`:
//! navigation rows (`NAV`), symbol rows (`ASSET`), ticker-function hints
//! (`FUNC`), command rows (`CMD`), and an "open market page" row for a bare
//! ticker. Rows are scored with [`fuzzy_score`] and sorted best first.

use crate::router::paths::ROUTES;
use crate::shared::command::commanding::{COMMAND_FUNCTIONS, fuzzy_score, looks_like_ticker, ticker_hints};
use crate::shared::command::symbols;

/// What a palette row does when chosen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaletteTarget {
    /// Navigate straight to a route.
    Route(String),
    /// Run this text through the command grammar (`RELIANCE DES`, `PORT`, ...).
    Command(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteItem {
    pub key: String,
    pub label: String,
    pub description: String,
    /// `NAV`, `ASSET`, `FUNC` or `CMD`.
    pub badge: &'static str,
    pub target: PaletteTarget,
    pub score: i32,
}

/// Navigation destinations: label, route, breadcrumb.
pub const NAV_ENTRIES: &[(&str, &str, &str)] = &[
    ("Portfolio", ROUTES::PORTFOLIO, "Workspace > Portfolio"),
    ("Screener", ROUTES::SCREENER, "Workspace > Screener"),
    ("Workbench", ROUTES::WORKBENCH, "Workspace > Workbench"),
    ("Strategy Studio", ROUTES::STRATEGY, "Workspace > Strategy"),
    ("Simulator", ROUTES::SIMULATOR, "Workspace > Simulator"),
    ("Tradebook", ROUTES::TRADEBOOK, "Workspace > Tradebook"),
    ("Security Hub", "/equity/security", "Research > Security Hub"),
    ("Research Dossiers", "/equity/research-dossiers", "Research > Dossiers"),
    ("Settings", ROUTES::SETTINGS, "System > Settings"),
    ("Funds", ROUTES::FUNDS, "System > Funds"),
];

/// Build and rank palette rows for `query`. An empty query lists navigation
/// first, then commands.
pub fn build_items(query: &str) -> Vec<PaletteItem> {
    let q = query.trim();
    let mut rows: Vec<PaletteItem> = Vec::new();

    for (idx, (label, route, crumb)) in NAV_ENTRIES.iter().enumerate() {
        let score = if q.is_empty() {
            1700 - idx as i32
        } else {
            fuzzy_score(label, q).max(fuzzy_score(route, q)).max(fuzzy_score(crumb, q))
        };
        if score >= 0 {
            rows.push(PaletteItem {
                key: format!("nav:{route}"),
                label: label.to_string(),
                description: crumb.to_string(),
                badge: "NAV",
                target: PaletteTarget::Route(route.to_string()),
                score,
            });
        }
    }

    if !q.is_empty() {
        for (idx, symbol) in symbols::search(q, 6).into_iter().enumerate() {
            rows.push(PaletteItem {
                key: format!("asset:{}", symbol.ticker),
                label: symbol.ticker.to_string(),
                description: format!("{} · {}", symbol.name, symbol.exchange),
                badge: "ASSET",
                target: PaletteTarget::Command(symbol.ticker.to_string()),
                score: 2100 - idx as i32,
            });
        }

        for (idx, hint) in ticker_hints(q).into_iter().enumerate() {
            rows.push(PaletteItem {
                key: hint.key,
                label: hint.title,
                description: hint.subtitle,
                badge: "FUNC",
                target: PaletteTarget::Command(hint.command),
                score: 1800 - idx as i32,
            });
        }
    }

    for spec in COMMAND_FUNCTIONS {
        let score = if q.is_empty() {
            1
        } else {
            [spec.code, spec.label, spec.description]
                .into_iter()
                .chain(spec.aliases.iter().copied())
                .map(|text| fuzzy_score(text, q))
                .max()
                .unwrap_or(-1)
        };
        if score >= 0 {
            rows.push(PaletteItem {
                key: format!("fn:{}", spec.code),
                label: spec.code.to_string(),
                description: spec.description.to_string(),
                badge: "CMD",
                target: PaletteTarget::Command(spec.code.to_string()),
                score,
            });
        }
    }

    if !q.is_empty() && looks_like_ticker(&q.to_ascii_uppercase())
        && !rows.iter().any(|r| r.badge == "ASSET")
    {
        let upper = q.to_ascii_uppercase();
        rows.push(PaletteItem {
            key: format!("ticker:{upper}"),
            label: upper.clone(),
            description: "Open market page".into(),
            badge: "CMD",
            target: PaletteTarget::Command(upper),
            score: 2000,
        });
    }

    rows.sort_by(|a, b| b.score.cmp(&a.score));
    rows.truncate(40);
    rows
}
