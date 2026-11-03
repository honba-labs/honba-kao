//! Deterministic, local natural-language prompt parsing for screener filters.
//!
//! This module is intentionally self-contained: it performs **no** `honba_nou`
//! linkage, no model calls and no I/O. The same input text always yields an
//! identical [`PromptAst`], which [`apply_prompt`] folds into a
//! [`FilterCriteria`].

use rust_decimal::Decimal;

use super::indices::benchmark_for;
use super::types::FilterCriteria;

/// A single recognized filter constraint extracted from a prompt clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptTerm {
    /// Inclusive lower bound on price (`min_price`).
    PriceMin(Decimal),
    /// Inclusive upper bound on price (`max_price`).
    PriceMax(Decimal),
    /// Inclusive lower bound on the P/E ratio (`min_pe`).
    PeMin(Decimal),
    /// Inclusive upper bound on the P/E ratio (`max_pe`).
    PeMax(Decimal),
    /// Inclusive lower bound on relative volume (`min_rvol`).
    RvolMin(Decimal),
    /// Inclusive lower bound on RSI (`min_rsi`).
    RsiMin(Decimal),
    /// Inclusive upper bound on RSI (`max_rsi`).
    RsiMax(Decimal),
    /// Membership in the index catalog entry `key` (`market_index`).
    Index(&'static str),
    /// "Futures & options only" flag (`fno_only`).
    FnoOnly,
    /// "Watchlist only" flag (`watchlist_only`).
    WatchlistOnly,
}

/// Result of parsing a prompt: ordered recognized terms plus raw leftover text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PromptAst {
    /// Recognized constraints, in clause order.
    pub terms: Vec<PromptTerm>,
    /// Unrecognized words (space-joined); ignored when `terms` is non-empty.
    pub residual: String,
}

/// Parse `text` into a [`PromptAst`].
///
/// Input is split on `,` and `;` into clauses; each clause becomes zero or more
/// [`PromptTerm`]s (boolean → index → numeric, in that order). Unrecognized
/// clauses are joined into `residual`; when nothing at all was recognized the
/// residual keeps the full original trimmed text (search fallback).
pub fn parse_prompt(text: &str) -> PromptAst {
    let mut terms = Vec::new();
    let mut residual_parts: Vec<&str> = Vec::new();
    for raw in text.split([',', ';']) {
        let clause = raw.trim();
        if clause.is_empty() {
            continue;
        }
        let parsed = parse_clause(clause);
        if parsed.is_empty() {
            residual_parts.push(clause);
        } else {
            terms.extend(parsed);
        }
    }
    let mut residual = residual_parts.join(" ");
    if terms.is_empty() && !residual.is_empty() {
        residual = text.trim().to_string();
    }
    PromptAst { terms, residual }
}

/// Fold a parsed prompt into a fresh [`FilterCriteria`] (defaults + terms).
///
/// When no terms were recognized and a residual exists, the residual becomes
/// the `search_query` (free-text search fallback).
pub fn apply_prompt(text: &str) -> FilterCriteria {
    let ast = parse_prompt(text);
    let mut criteria = FilterCriteria::default();
    if ast.terms.is_empty() {
        criteria.search_query = ast.residual;
        return criteria;
    }
    for term in ast.terms {
        match term {
            PromptTerm::PriceMin(value) => criteria.min_price = value,
            PromptTerm::PriceMax(value) => criteria.max_price = value,
            PromptTerm::PeMin(value) => criteria.min_pe = value,
            PromptTerm::PeMax(value) => criteria.max_pe = value,
            PromptTerm::RvolMin(value) => criteria.min_rvol = value,
            PromptTerm::RsiMin(value) => criteria.min_rsi = value,
            PromptTerm::RsiMax(value) => criteria.max_rsi = value,
            PromptTerm::Index(key) => criteria.market_index = key.to_string(),
            PromptTerm::FnoOnly => criteria.fno_only = true,
            PromptTerm::WatchlistOnly => criteria.watchlist_only = true,
        }
    }
    criteria
}

/// Render a human-readable chip label for `text` (`" · "`-joined terms).
///
/// Empty input yields `""`; a residual-only prompt yields `"Search: {residual}"`.
pub fn prompt_label(text: &str) -> String {
    let ast = parse_prompt(text);
    if ast.terms.is_empty() {
        if ast.residual.is_empty() {
            String::new()
        } else {
            format!("Search: {}", ast.residual)
        }
    } else {
        ast.terms
            .iter()
            .map(|term| match term {
                PromptTerm::PriceMin(value) => format!("Price>{value}"),
                PromptTerm::PriceMax(value) => format!("Price<{value}"),
                PromptTerm::PeMin(value) => format!("PE>{value}"),
                PromptTerm::PeMax(value) => format!("PE<{value}"),
                PromptTerm::RvolMin(value) => format!("RVOL>{value}"),
                PromptTerm::RsiMin(value) => format!("RSI>{value}"),
                PromptTerm::RsiMax(value) => format!("RSI<{value}"),
                PromptTerm::Index(key) => benchmark_for(key)
                    .map(|benchmark| benchmark.label.to_string())
                    .unwrap_or_default(),
                PromptTerm::FnoOnly => "F&O".to_string(),
                PromptTerm::WatchlistOnly => "Watchlist".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

/// Parse one trimmed clause into zero or more terms; empty ⇒ residual.
fn parse_clause(raw: &str) -> Vec<PromptTerm> {
    let trimmed = raw.trim();
    // Boolean exact-clause matches (case-insensitive).
    match trimmed.to_lowercase().as_str() {
        "fno" | "f&o" | "fno only" => return vec![PromptTerm::FnoOnly],
        "watchlist" | "watchlist only" | "stared" | "star" => {
            return vec![PromptTerm::WatchlistOnly];
        }
        _ => {}
    }
    // Index aliases (every non-alphanumeric stripped), confirmed against the catalog.
    let alias = index_alias(trimmed).filter(|key| benchmark_for(key).is_some());
    if let Some(key) = alias {
        return vec![PromptTerm::Index(key)];
    }
    // Fall through to the numeric branch (aliases may carry numeric text).
    numeric_clause(trimmed)
}

/// Map a clause to a static index key by stripping all non-alphanumeric chars.
fn index_alias(clause: &str) -> Option<&'static str> {
    let flat: String = clause
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    match flat.as_str() {
        "nifty50" => Some("NIFTY50"),
        "niftynext50" => Some("NIFTYNEXT50"),
        "nifty100" => Some("NIFTY100"),
        "nifty200" => Some("NIFTY200"),
        "nifty500" => Some("NIFTY500"),
        "niftymidcap150" | "midcap150" => Some("NIFTYMIDCAP150"),
        "niftysmallcap250" | "smallcap250" => Some("NIFTYSMALLCAP250"),
        "niftybank" | "banknifty" => Some("NIFTYBANK"),
        "niftyit" => Some("NIFTYIT"),
        "niftyauto" => Some("NIFTYAUTO"),
        "niftypharma" => Some("NIFTYPHARMA"),
        _ => None,
    }
}

/// Numeric field selector recognized as the first token of a clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericField {
    /// `price` (alias `ltp`).
    Price,
    /// `pe`.
    Pe,
    /// `rvol`.
    Rvol,
    /// `rsi`.
    Rsi,
}

impl NumericField {
    /// Map a raw first token to a field; `ltp` behaves like `price`.
    fn from_token(token: &str) -> Option<Self> {
        match token {
            "price" | "ltp" => Some(Self::Price),
            "pe" => Some(Self::Pe),
            "rvol" => Some(Self::Rvol),
            "rsi" => Some(Self::Rsi),
            _ => None,
        }
    }
}

/// Direction of a relational operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Relation {
    /// Lower bound (`>`, `>=`, `over`, `above`, `min`).
    Lower,
    /// Upper bound (`<`, `<=`, `under`, `below`, `max`).
    Upper,
}

/// Map a relation token to [`Relation`]; `None` means "not a relation".
fn rel_from_token(token: &str) -> Option<Relation> {
    match token {
        "<" | "<=" | "under" | "below" | "max" => Some(Relation::Upper),
        ">" | ">=" | "over" | "above" | "min" => Some(Relation::Lower),
        _ => None,
    }
}

/// Lowercased, whitespace-collapsed token stream with `<`, `>`, `<=`, `>=`
/// padded by single spaces so relational clauses tokenize cleanly. `<=`/`>=`
/// are recognized via one-character lookahead before bare `<`/`>` spacing.
fn numeric_stream(raw: &str) -> String {
    let collapsed = raw
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut spaced = String::with_capacity(collapsed.len() + 8);
    let mut chars = collapsed.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '<' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    spaced.push_str(" <= ");
                } else {
                    spaced.push_str(" < ");
                }
            }
            '>' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    spaced.push_str(" >= ");
                } else {
                    spaced.push_str(" > ");
                }
            }
            _ => spaced.push(c),
        }
    }
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parse one numeric token: strips `₹`, `,`, `%` and a trailing `x`, then uses
/// exact decimal parsing (no rounding, zero floats).
fn parse_amount(token: &str) -> Option<Decimal> {
    let cleaned: String = token
        .chars()
        .filter(|c| !matches!(*c, '₹' | ',' | '%'))
        .collect();
    let trimmed = cleaned.trim();
    let bare = trimmed
        .strip_suffix('x')
        .or_else(|| trimmed.strip_suffix('X'))
        .unwrap_or(trimmed);
    if bare.is_empty() {
        return None;
    }
    Decimal::from_str_exact(bare).ok()
}

/// Build the bound terms for a `min..max` range, swapping on reversal.
fn range_terms(field: NumericField, low: &str, high: &str) -> Vec<PromptTerm> {
    let (Some(a), Some(b)) = (parse_amount(low), parse_amount(high)) else {
        return Vec::new();
    };
    let (min, max) = if a > b { (b, a) } else { (a, b) };
    match field {
        NumericField::Price => vec![PromptTerm::PriceMin(min), PromptTerm::PriceMax(max)],
        NumericField::Pe => vec![PromptTerm::PeMin(min), PromptTerm::PeMax(max)],
        NumericField::Rsi => vec![PromptTerm::RsiMin(min), PromptTerm::RsiMax(max)],
        // The frozen contract has no `RvolMax`; only the lower bound applies.
        NumericField::Rvol => vec![PromptTerm::RvolMin(min)],
    }
}

/// Build the single bound term for `field rel value`.
fn bound_terms(field: NumericField, relation: Relation, value: Decimal) -> Vec<PromptTerm> {
    match relation {
        Relation::Lower => match field {
            NumericField::Price => vec![PromptTerm::PriceMin(value)],
            NumericField::Pe => vec![PromptTerm::PeMin(value)],
            NumericField::Rvol => vec![PromptTerm::RvolMin(value)],
            NumericField::Rsi => vec![PromptTerm::RsiMin(value)],
        },
        Relation::Upper => match field {
            NumericField::Price => vec![PromptTerm::PriceMax(value)],
            NumericField::Pe => vec![PromptTerm::PeMax(value)],
            NumericField::Rsi => vec![PromptTerm::RsiMax(value)],
            // No `RvolMax` exists: reject the clause as residual.
            NumericField::Rvol => Vec::new(),
        },
    }
}

/// Parse the numeric branch of a clause; empty vec ⇒ residual (whole raw clause).
///
/// Greedy rule: a complete numeric assignment consumes the clause — whitespace
/// tokens after it (`"and"`, `"moon"`) are silently dropped. The clause only
/// becomes residual when the first two tokens already fail to form a valid
/// `field rel number` / `field a-b` / range pattern, or the number is unparsable.
fn numeric_clause(raw: &str) -> Vec<PromptTerm> {
    let stream = numeric_stream(raw);
    let tokens: Vec<&str> = stream.split_whitespace().collect();
    let Some(field) = tokens.first().copied().and_then(NumericField::from_token) else {
        return Vec::new();
    };
    // [field, "between", a, "and", b]
    if tokens.len() >= 5 && tokens[1] == "between" && tokens[3] == "and" {
        return range_terms(field, tokens[2], tokens[4]);
    }
    // [field, a, "to", b]
    if tokens.len() >= 4 && tokens[2] == "to" {
        return range_terms(field, tokens[1], tokens[3]);
    }
    // [field, rel, num] — trailing junk after the assignment is ignored.
    let relation = if tokens.len() >= 3 {
        rel_from_token(tokens[1])
    } else {
        None
    };
    if let Some(relation) = relation {
        // Recognized field + relation but an unparsable number ⇒ residual.
        let Some(value) = parse_amount(tokens[2]) else {
            return Vec::new();
        };
        return bound_terms(field, relation, value);
    }
    // [field, "a-b"] — one token whose two halves both parse as decimals.
    let dashed = if tokens.len() == 2 {
        tokens[1]
            .split_once('-')
            .filter(|(low, high)| !low.is_empty() && !high.is_empty())
    } else {
        None
    };
    if let Some((low, high)) = dashed {
        return range_terms(field, low, high);
    }
    // Bare number, missing relation, unknown field or malformed input ⇒ residual.
    Vec::new()
}
