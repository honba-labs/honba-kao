//! Inspector panel: shows param inputs from catalogue ranges.
//! Displays the parameter fields for a selected node kind, built from the
//! catalogue whitelist. Changes update the graph via store actions.

use leptos::prelude::*;

// Catalogue ranges (min/max/step) for each indicator + transform ops.
// These mirror the whitelist in `palette.rs` and the desk's `catalogue.rs`.
fn indicator_ranges(ind: &str) -> (&'static str, &'static str, &'static str) {
    // (min, max, step) as strings for the HTML input attributes
    match ind {
        "Sma" => ("0.1", "50.0", "0.1"),
        "Ema" => ("0.1", "50.0", "0.1"),
        "Rsi" => ("10.0", "90.0", "1.0"),
        "Macd" => ("fast: 12, slow: 26, signal: 9".to_string(), "".to_string(), "".to_string()),
        "Bollinger" => ("2.0", "100.0", "0.2"),
        "Donchian" => ("10.0", "200.0", "1.0"),
        "Attr" => ("0.1", "10.0", "0.1"),
        "Stochastic" => ("14.0", "14.0", "1.0"), // %K period
        "WilliamsR" => ("-80.0", "20.0", "1.0"),
        _ => ("0.0", "100.0", "1.0"),
    }
}

/// Inspector panel parameter fields for a selected node.
#[component]
pub fn NodeInspector {
    /// The node kind whose params we display.
    #[prop(into)]
    kind: String,
}

let kind = kind;
let (min_val, max_val, step) = indicator_ranges(&kind);

view! {
    <div class="node-inspector">
        <h3>Params for {kind}</h3>
        <div class="param-row">
            <label>Min<br>
                <input type="number" min={min_val} step={step} style="width: 80px;"/>
            </label>
        </div>
        <div class="param-row">
            <label>Max<br>
                <input type="number" max={max_val} step={step} style="width: 80px;"/>
            </label>
        </div>
        <div class="param-row">
            <label>Step<br>
                <input type="number" step={step} style="width: 80px;"/>
            </label>
        </div>
    </div>
}
