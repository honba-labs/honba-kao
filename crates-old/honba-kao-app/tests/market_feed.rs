//! Quote book: synthetic seeding, live frame application, provenance fallback.

use honba_kao_app::shared::command::symbols::MOCK_SYMBOLS;
use honba_kao_app::shared::stream::feed::QuoteBook;
use honba_kao_client::stream::{Provenance, Quality, StreamFrame, parse_frame};
use honba_kao_ui_core::trading::ProvenanceQuality;

fn quote(symbol: &str, ltp: &str, quality: Quality) -> StreamFrame {
    StreamFrame::Quote {
        symbol: symbol.into(),
        ltp: ltp.into(),
        change_pct: "0.5".into(),
        provenance: Provenance {
            quality,
            source: "kite".into(),
            note: None,
            as_of: None,
            latency_ms: None,
        },
    }
}

#[test]
fn seeding_gives_every_mock_symbol_a_synthetic_quote() {
    let mut book = QuoteBook::new();
    book.seed_synthetic(MOCK_SYMBOLS);
    let reliance = book.quote("RELIANCE").expect("seeded");
    assert_eq!(reliance.quality, ProvenanceQuality::Synthetic);
    assert_eq!(reliance.source, "mock");
    assert_eq!(book.overall_quality(), ProvenanceQuality::Synthetic);
}

#[test]
fn live_frame_replaces_synthetic_and_lifts_feed_to_live() {
    let mut book = QuoteBook::new();
    book.seed_synthetic(MOCK_SYMBOLS);
    assert!(book.apply_frame(&quote("RELIANCE", "3001.00", Quality::Live)));
    let q = book.quote("RELIANCE").unwrap();
    assert_eq!(q.ltp, "3001.00");
    assert_eq!(q.quality, ProvenanceQuality::Live);
    assert_eq!(book.overall_quality(), ProvenanceQuality::Live);
}

#[test]
fn identical_frame_reports_no_change() {
    let mut book = QuoteBook::new();
    let frame = quote("TCS", "4000.00", Quality::Live);
    assert!(book.apply_frame(&frame));
    assert!(!book.apply_frame(&frame));
}

#[test]
fn seeding_does_not_overwrite_live_quotes() {
    let mut book = QuoteBook::new();
    book.apply_frame(&quote("INFY", "1700.00", Quality::Live));
    book.seed_synthetic(MOCK_SYMBOLS);
    assert_eq!(book.quote("INFY").unwrap().ltp, "1700.00");
}

#[test]
fn going_offline_downgrades_live_quotes_to_cached() {
    let mut book = QuoteBook::new();
    book.seed_synthetic(MOCK_SYMBOLS);
    book.apply_frame(&quote("INFY", "1700.00", Quality::Live));
    book.mark_offline();
    assert_eq!(
        book.quote("INFY").unwrap().quality,
        ProvenanceQuality::Cached
    );
    assert_eq!(
        book.quote("TCS").unwrap().quality,
        ProvenanceQuality::Synthetic
    );
    assert_eq!(book.overall_quality(), ProvenanceQuality::Cached);
}

#[test]
fn non_quote_frames_are_ignored() {
    let mut book = QuoteBook::new();
    let heartbeat = parse_frame(r#"{"type":"heartbeat","ts":"t"}"#).unwrap();
    assert!(!book.apply_frame(&heartbeat));
}
