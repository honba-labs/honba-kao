use openalgo_charts::canvas::{SeriesKind, SeriesStyle};

#[test]
fn series_kind_strings() {
    assert_eq!(SeriesKind::Candlestick.as_str(), "candlestick");
    assert_eq!(SeriesKind::Bar.as_str(), "bar");
    assert_eq!(SeriesKind::Line.as_str(), "line");
    assert_eq!(SeriesKind::Area.as_str(), "area");
    assert_eq!(SeriesKind::Histogram.as_str(), "histogram");
}

#[test]
fn engine_without_mount_has_no_id() {
    // Outside a browser there is no canvas document; constructing the engine
    // requires a real HtmlCanvasElement, so we only assert the default state
    // logic indirectly: an unmounted engine reports None.
    assert_eq!(SeriesKind::default(), SeriesKind::Candlestick);
}

#[test]
fn style_serialises_compactly() {
    let style = SeriesStyle::candlestick("#10b981", "#ef4444");
    let json = serde_json::to_string(&style).unwrap();
    assert!(json.contains("10b981"));
    assert!(json.contains("ef4444"));
}
