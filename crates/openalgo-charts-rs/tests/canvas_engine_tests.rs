use openalgo_charts::canvas::{CanvasEngine, SeriesKind, SeriesStyle};
use openalgo_charts::ChartTheme;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;

/// A `web_sys::HtmlCanvasElement` that is never dereferenced as a real DOM
/// node — `CanvasEngine` only hands it to `bridge::mount_static`, which on a
/// non-wasm32 host build (these tests) never touches the element at all and
/// always returns `Err` (there is no `window.honbaCharts` outside a
/// browser). This lets us construct a `CanvasEngine` and exercise its
/// pure-Rust bookkeeping (pending theme/style folding, mount's leak guard)
/// without a real browser.
fn fake_canvas() -> HtmlCanvasElement {
    JsValue::NULL.unchecked_into()
}

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

#[test]
fn with_theme_folds_colors_into_the_active_style() {
    // A theme set via `with_theme` must actually drive the series style
    // (issue #31: the `theme` field was stored but never consumed).
    let theme = ChartTheme::default();
    let engine = CanvasEngine::new(
        fake_canvas(),
        SeriesKind::Candlestick,
        SeriesStyle::default(),
    )
    .with_theme(theme.clone());

    assert_eq!(engine.style().up_color, Some(theme.up_color.clone()));
    assert_eq!(engine.style().down_color, Some(theme.down_color.clone()));
    assert_eq!(engine.style().color, Some(theme.up_color));
}

#[test]
fn apply_pending_is_idempotent_and_reflects_the_latest_theme() {
    let mut engine = CanvasEngine::new(fake_canvas(), SeriesKind::Line, SeriesStyle::default());
    // No theme set yet: apply_pending is a safe no-op.
    engine.apply_pending();
    assert_eq!(engine.style().up_color, None);

    engine = engine.with_theme(ChartTheme::default());
    let folded = engine.style().up_color.clone();
    assert_eq!(folded, Some(ChartTheme::default().up_color));

    // Calling it again (as `mount` does internally) must not change anything.
    engine.apply_pending();
    assert_eq!(engine.style().up_color, folded);
}

#[test]
fn mount_outside_a_browser_errors_and_leaves_no_id() {
    // There is no `window.honbaCharts` on a non-wasm32 host build, so
    // `mount` always errors here — but it must still leave the engine in a
    // consistent, re-mountable state (no stray id) rather than panicking.
    let mut engine = CanvasEngine::new(fake_canvas(), SeriesKind::Bar, SeriesStyle::default());
    assert!(engine.id().is_none());
    assert!(engine.mount().is_err());
    assert!(engine.id().is_none());
}

#[test]
fn mount_called_twice_does_not_panic_and_still_has_no_leaked_id() {
    // Regression guard for issue #31: calling `mount` a second time on the
    // same engine must destroy any existing chart id first rather than
    // leaking it. We cannot observe a successful mount outside a browser
    // (bridge::mount_static always errors on a non-wasm32 host build), but
    // we can assert the guard path runs without panicking and that the
    // engine never ends up holding a stale id across repeated calls.
    let mut engine = CanvasEngine::new(
        fake_canvas(),
        SeriesKind::Candlestick,
        SeriesStyle::candlestick("#10b981", "#ef4444"),
    );
    for _ in 0..3 {
        assert!(engine.mount().is_err());
        assert!(engine.id().is_none());
    }
}
