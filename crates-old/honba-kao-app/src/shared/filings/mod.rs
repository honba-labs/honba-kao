//! Filings citation search — a small, real, local lookup module.
//!
//! This module performs genuine, deterministic keyword/substring search over
//! a tiny checked-in corpus of sample filing-style text ([`documents`]), and
//! returns the best-matching verbatim sentence together with its document
//! name and page number. It backs the "scored drivers" citations and the
//! "Ask the Filings" QA panel in `features::security_hub::filings_tab`.
//!
//! **What this is**: exact/fuzzy word-overlap search over a few checked-in
//! pages, run live against whatever query string it is given. Every
//! `(document, page_number, quote)` triple returned by [`search`] is read
//! directly out of the page text in [`documents`] — nothing is pre-baked
//! per-question.
//!
//! **What this is NOT**: a production RAG pipeline. There is no ingestion,
//! no embeddings, no vector index, no LLM-driven relevance or driver
//! scoring, and the corpus is three hand-written sample documents, not a
//! live filings feed. The driver *labels and scores* shown in the Filings
//! tab remain a small hand-authored list (see `filings_tab.rs`) — only the
//! citation attached to each one is a real lookup result from this module.
//! The tracked work to replace this with a real ingestion + RAG backend
//! lives in `honba-nou`, `honba-data`, and `honba-tamashii` (see
//! honba-nou#1 / honba-data#1 / honba-tamashii#1 on GitHub), not here.

pub mod documents;

/// One page of a checked-in sample document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilingPage {
    pub document: &'static str,
    pub page_number: u32,
    pub text: &'static str,
}

/// A verbatim search result: the exact sentence matched, and where it came
/// from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Citation {
    pub document: String,
    pub page_number: u32,
    pub quote: String,
}

/// Lowercases and strips punctuation from a single word for matching.
fn normalize_word(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

/// Splits a query into normalized keywords, dropping very short filler
/// words (e.g. "is", "of") so matches are driven by meaningful terms.
fn query_words(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(normalize_word)
        .filter(|w| w.len() >= 3)
        .collect()
}

/// Splits page text into trimmed sentences on `.`/`!`/`?`, preserving the
/// original (verbatim) casing and wording of each sentence.
fn split_sentences(text: &str) -> Vec<&str> {
    let mut sentences = Vec::new();
    let mut start = 0;
    for (byte_idx, ch) in text.char_indices() {
        if ch == '.' || ch == '!' || ch == '?' {
            let end = byte_idx + ch.len_utf8();
            let candidate = text[start..end].trim();
            if !candidate.is_empty() {
                sentences.push(candidate);
            }
            start = end;
        }
    }
    let remainder = text[start..].trim();
    if !remainder.is_empty() {
        sentences.push(remainder);
    }
    sentences
}

/// Counts how many distinct query keywords appear as a substring of the
/// (lowercased) sentence.
fn sentence_score(sentence: &str, keywords: &[String]) -> usize {
    let lowered = sentence.to_lowercase();
    keywords
        .iter()
        .filter(|keyword| lowered.contains(keyword.as_str()))
        .count()
}

/// Runs a real, deterministic keyword/substring search for `query` over the
/// checked-in sample corpus in [`documents`] and returns the highest-scoring
/// verbatim sentence as a [`Citation`], or `None` if no page contains any of
/// the query's keywords.
///
/// This is intentionally simple (no stemming, no ranking model, no index) —
/// it is a real search, not a canned answer map, but it is also not a claim
/// of semantic/RAG-quality retrieval.
pub fn search(query: &str) -> Option<Citation> {
    let keywords = query_words(query);
    if keywords.is_empty() {
        return None;
    }

    let mut best: Option<(usize, Citation)> = None;
    for page in documents::all_pages() {
        for sentence in split_sentences(page.text) {
            let score = sentence_score(sentence, &keywords);
            if score == 0 {
                continue;
            }
            let is_better = match &best {
                Some((best_score, _)) => score > *best_score,
                None => true,
            };
            if is_better {
                best = Some((
                    score,
                    Citation {
                        document: page.document.to_string(),
                        page_number: page.page_number,
                        quote: sentence.to_string(),
                    },
                ));
            }
        }
    }

    best.map(|(_, citation)| citation)
}
