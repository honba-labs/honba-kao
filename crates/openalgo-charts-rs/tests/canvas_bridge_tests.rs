use openalgo_charts::canvas::bridge::*;

#[test]
fn honba_charts_absent_is_a_graceful_noop() {
    // Outside a browser `window.honbaCharts` is not defined; calls must not
    // panic, they return an error the engine can degrade from.
    assert!(set_series(0.0, "candlestick", "{}", "[]").is_err());
    assert!(update_last(0.0, "{}").is_err());
    assert!(destroy(0.0).is_err());
    assert!(refresh_theme_all().is_err());

    // `mount_static` needs a real canvas element and a browser DOM; only
    // meaningful on wasm32.
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Ok(canvas) = doc.create_element("canvas") {
                let el: web_sys::HtmlElement = canvas
                    .dyn_into::<web_sys::HtmlElement>()
                    .expect("canvas is an HtmlElement");
                assert!(mount_static(&el).is_err());
            }
        }
    }
}
