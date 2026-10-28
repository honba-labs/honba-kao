//! CSS token bridging and theme generation for OpenAlgo Charts.

use crate::types::ChartTheme;

/// Read a CSS custom property (e.g. `"--pnl-gain"`) off the active DOM element.
pub fn css_var(name: &str, fallback: &str) -> String {
    let Some(window) = web_sys::window() else {
        return fallback.to_string();
    };
    let Some(document) = window.document() else {
        return fallback.to_string();
    };
    let host = document
        .body()
        .map(|b| b.into())
        .or_else(|| document.document_element());

    let Some(host) = host else {
        return fallback.to_string();
    };

    let Ok(computed) = window.get_computed_style(&host) else {
        return fallback.to_string();
    };
    let Some(computed) = computed else {
        return fallback.to_string();
    };

    let val = computed.get_property_value(name).unwrap_or_default();
    let trimmed = val.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Detect whether the current document is in dark mode (via `data-mode="dark"` or system preference).
pub fn is_dark_mode() -> bool {
    let Some(window) = web_sys::window() else {
        return true;
    };
    let Some(document) = window.document() else {
        return true;
    };
    if let Some(root) = document.document_element() {
        if let Some(mode) = root.get_attribute("data-mode") {
            return mode != "light";
        }
    }
    true
}

/// Compose a [`ChartTheme`] by sampling the active CSS design tokens.
pub fn theme_from_skin() -> ChartTheme {
    let base = if is_dark_mode() {
        ChartTheme::dark()
    } else {
        ChartTheme::light()
    };

    ChartTheme {
        background: css_var("--chart-background", &base.background),
        grid: css_var("--chart-grid", &base.grid),
        axis_text: css_var("--chart-axis-text", &base.axis_text),
        axis_line: css_var("--chart-axis-line", &base.axis_line),
        pane_separator: css_var("--chart-pane-separator", &base.pane_separator),
        crosshair: css_var("--chart-crosshair", &base.crosshair),

        up_color: css_var("--pnl-gain", &base.up_color),
        down_color: css_var("--pnl-loss", &base.down_color),
        wick_up_color: css_var("--pnl-gain", &base.wick_up_color),
        wick_down_color: css_var("--pnl-loss", &base.wick_down_color),

        line_color: css_var("--primary-8", &base.line_color),
        area_top_color: css_var("--chart-area-top", &base.area_top_color),
        area_bottom_color: css_var("--chart-area-bottom", &base.area_bottom_color),

        buy: css_var("--pnl-gain", &base.buy),
        sell: css_var("--pnl-loss", &base.sell),
        profit: css_var("--pnl-gain", &base.profit),
        loss: css_var("--pnl-loss", &base.loss),
    }
}

/// Resolve a color specification which might be a literal hex/rgb string or a CSS token like `"--primary-8"`.
pub fn resolve_color(value: &str, fallback: &str) -> String {
    if value.starts_with("--") {
        css_var(value, fallback)
    } else if !value.is_empty() {
        value.to_string()
    } else {
        fallback.to_string()
    }
}
