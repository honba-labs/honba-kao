//! Asset integrity tests for `honba-kao-assets`.
//!
//! Every byte pinned here is pulled in via `include_bytes!` /
//! `include_str!` with test-relative `../assets/...` paths — no runtime
//! filesystem access, no absolute paths.

use std::collections::HashSet;

use honba_kao_assets::audio::SoundCue;
use honba_kao_assets::{branding, fixtures, fonts, themes};
use nsh_base::Theme;

const TTF_MAGIC: &[u8] = b"\x00\x01\x00\x00";
const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
const ICO_MAGIC: &[u8] = b"\x00\x00\x01\x00";

fn ttf_files() -> Vec<(&'static str, &'static [u8])> {
    vec![
        (
            "fonts/Exo2-Regular.ttf",
            &include_bytes!("../assets/fonts/Exo2-Regular.ttf")[..],
        ),
        (
            "fonts/Exo2-SemiBold.ttf",
            &include_bytes!("../assets/fonts/Exo2-SemiBold.ttf")[..],
        ),
        (
            "fonts/Exo2-Bold.ttf",
            &include_bytes!("../assets/fonts/Exo2-Bold.ttf")[..],
        ),
        (
            "fonts/Exo2-ExtraBold.ttf",
            &include_bytes!("../assets/fonts/Exo2-ExtraBold.ttf")[..],
        ),
        (
            "fonts/RobotoMono-Regular.ttf",
            &include_bytes!("../assets/fonts/RobotoMono-Regular.ttf")[..],
        ),
        (
            "fonts/RobotoMono-Bold.ttf",
            &include_bytes!("../assets/fonts/RobotoMono-Bold.ttf")[..],
        ),
    ]
}

fn wav_files() -> Vec<(&'static str, &'static [u8])> {
    vec![
        (
            "audio/order_submitted.wav",
            &include_bytes!("../assets/audio/order_submitted.wav")[..],
        ),
        (
            "audio/order_filled.wav",
            &include_bytes!("../assets/audio/order_filled.wav")[..],
        ),
        (
            "audio/order_cancelled.wav",
            &include_bytes!("../assets/audio/order_cancelled.wav")[..],
        ),
        (
            "audio/order_rejected.wav",
            &include_bytes!("../assets/audio/order_rejected.wav")[..],
        ),
        (
            "audio/margin_warning.wav",
            &include_bytes!("../assets/audio/margin_warning.wav")[..],
        ),
        (
            "audio/price_alert.wav",
            &include_bytes!("../assets/audio/price_alert.wav")[..],
        ),
        (
            "audio/ping_connected.wav",
            &include_bytes!("../assets/audio/ping_connected.wav")[..],
        ),
    ]
}

fn png_files() -> Vec<(&'static str, &'static [u8])> {
    vec![
        (
            "branding/logo@2x.png",
            &include_bytes!("../assets/branding/logo@2x.png")[..],
        ),
        (
            "branding/logo-monochrome@2x.png",
            &include_bytes!("../assets/branding/logo-monochrome@2x.png")[..],
        ),
        (
            "branding/app-icon.png",
            &include_bytes!("../assets/branding/app-icon.png")[..],
        ),
        (
            "emblems/exchanges/nse.png",
            &include_bytes!("../assets/emblems/exchanges/nse.png")[..],
        ),
        (
            "emblems/exchanges/bse.png",
            &include_bytes!("../assets/emblems/exchanges/bse.png")[..],
        ),
        (
            "emblems/exchanges/mcx.png",
            &include_bytes!("../assets/emblems/exchanges/mcx.png")[..],
        ),
        (
            "emblems/brokers/zerodha.png",
            &include_bytes!("../assets/emblems/brokers/zerodha.png")[..],
        ),
        (
            "emblems/brokers/interactive_brokers.png",
            &include_bytes!("../assets/emblems/brokers/interactive_brokers.png")[..],
        ),
        (
            "emblems/brokers/openalgo.png",
            &include_bytes!("../assets/emblems/brokers/openalgo.png")[..],
        ),
    ]
}

#[test]
fn ttf_magic_bytes_and_size_floor() {
    for (name, bytes) in ttf_files() {
        assert_eq!(&bytes[..4], TTF_MAGIC, "{name} must be a TrueType font");
        assert!(bytes.len() > 40_000, "{name} is suspiciously small");
    }
}

#[test]
fn wav_magic_bytes_and_size_floor() {
    for (name, bytes) in wav_files() {
        assert_eq!(&bytes[..4], b"RIFF", "{name} must be RIFF-framed");
        assert_eq!(&bytes[8..12], b"WAVE", "{name} must be a WAVE file");
        assert!(bytes.len() > 4_000, "{name} is suspiciously small");
    }
}

#[test]
fn png_magic_bytes_and_size_floor() {
    for (name, bytes) in png_files() {
        assert_eq!(&bytes[..8], PNG_MAGIC, "{name} must be a PNG");
        assert!(bytes.len() > 1_000, "{name} is suspiciously small");
    }
}

#[test]
fn favicon_ico_magic_bytes_and_size_floor() {
    let bytes = include_bytes!("../assets/branding/favicon.ico");
    assert_eq!(&bytes[..4], ICO_MAGIC, "favicon.ico must be an ICO");
    assert!(bytes.len() > 1_500, "favicon.ico is suspiciously small");
}

#[test]
fn branding_constants_smoke() {
    // Touch every branding constant so a rename/removal fails here first.
    let _ = [
        branding::LOGO_2X,
        branding::LOGO_MONO_2X,
        branding::APP_ICON,
        branding::exchanges::NSE,
        branding::exchanges::BSE,
        branding::exchanges::MCX,
        branding::brokers::ZERODHA,
        branding::brokers::INTERACTIVE_BROKERS,
        branding::brokers::OPENALGO,
    ];
    // `.ico` is a `FileAsset` (nsh's image set is png/jpg/jpeg/gif/webp/heic/svg).
    let _: nsh_core::element::FileAsset = branding::FAVICON;
}

#[test]
fn font_family_pins() {
    assert_eq!(fonts::EXO2.family, fonts::FAMILY_EXO2);
    assert_eq!(fonts::FAMILY_EXO2, "Exo 2");
    assert_eq!(fonts::MONO.family, fonts::FAMILY_MONO);
    assert_eq!(fonts::FAMILY_MONO, "Roboto Mono");
}

#[test]
fn themes_share_key_shape_and_parse_through_nsh() {
    for json in [
        themes::DEFAULT_THEME_JSON,
        themes::MONOKAI_THEME_JSON,
        themes::BLOOMBERG_THEME_JSON,
    ] {
        let value: serde_json::Value = serde_json::from_str(json).expect("theme JSON must parse");
        assert!(value["name"].is_string(), "theme must carry a name");
        let light = value["cssVars"]["light"]
            .as_object()
            .expect("cssVars.light must be an object");
        let dark = value["cssVars"]["dark"]
            .as_object()
            .expect("cssVars.dark must be an object");
        let light_keys: HashSet<&String> = light.keys().collect();
        let dark_keys: HashSet<&String> = dark.keys().collect();
        assert_eq!(light_keys, dark_keys, "light/dark key sets must match");

        // The REAL nsh parser must accept every shipped theme.
        assert!(
            Theme::from_json(json).is_ok(),
            "nsh_base::Theme::from_json must accept the theme"
        );
    }

    // default_theme() must not panic, and the name lookup table works.
    let _default = themes::default_theme();
    assert!(themes::theme_by_name("default").is_some());
    assert!(themes::theme_by_name("honba").is_some());
    assert!(themes::theme_by_name("monokai").is_some());
    assert!(themes::theme_by_name("bloomberg").is_some());
    assert!(themes::theme_by_name("nonsense").is_none());
}

#[test]
fn sound_cue_corpus_is_nonempty_and_distinct() {
    let cues = [
        SoundCue::OrderSubmitted,
        SoundCue::OrderFilled,
        SoundCue::OrderCancelled,
        SoundCue::OrderRejected,
        SoundCue::MarginWarning,
        SoundCue::PriceAlert,
        SoundCue::SystemConnected,
    ];
    let mut lengths = HashSet::new();
    for cue in cues {
        let bytes = cue.wav_bytes();
        assert!(!bytes.is_empty(), "{cue:?} must ship WAV bytes");
        assert!(bytes.len() > 4_000, "{cue:?} is suspiciously small");
        assert!(
            lengths.insert(bytes.len()),
            "{cue:?} duplicates another cue's length"
        );
    }
    assert_eq!(lengths.len(), 7, "all seven cues must be pairwise-distinct");
}

#[test]
fn sound_cue_hash_eq_and_match() {
    // Derives exist: the cue is hashable, equatable, and pattern-matchable.
    let cue = SoundCue::OrderFilled;
    let mut seen = HashSet::new();
    assert!(seen.insert(cue));
    assert!(!seen.insert(cue), "Eq must dedupe the same cue");
    match cue {
        SoundCue::OrderFilled => {}
        _ => panic!("match must resolve the OrderFilled variant"),
    }
}

#[test]
fn replay_header_and_first_record() {
    assert_eq!(fixtures::replay_tick_count(), Some(128));
    let bytes = fixtures::REPLAY_SAMPLE_TICKS;
    assert!(
        bytes.starts_with(fixtures::REPLAY_MAGIC),
        "stream must open with HBREPLAY1"
    );
    // First record starts at offset 14; side is the u8 at record offset +8.
    let first_record = 14;
    assert!(bytes.len() >= first_record + fixtures::REPLAY_RECORD_LEN);
    let side = bytes[first_record + 8];
    assert!((0..=2).contains(&side), "side must be bid/ask/trade");
}

#[test]
fn orderbook_snapshot_shape() {
    let value: serde_json::Value =
        serde_json::from_str(fixtures::ORDERBOOK_L2_SNAPSHOT_JSON).expect("fixture must parse");
    for side in ["bids", "asks"] {
        let levels = value[side].as_array().expect("bids/asks must be arrays");
        assert_eq!(levels.len(), 10, "{side} must carry 10 levels");
        for level in levels {
            let pair = level.as_array().expect("level must be [price, qty]");
            assert!(
                pair[0].is_string(),
                "price fields must be decimal strings (zero-float invariant)"
            );
        }
    }
}

#[test]
fn registry_smoke() {
    let reg = honba_kao_assets::registry();
    drop(reg);
}
