//! OpenScript IDE mock panel (Phase 4.2). A UI shell only: there is no
//! compiler or language runtime behind it. "Compile" fabricates diagnostics
//! so the editor, output list and indicator library can be exercised.

use honba_kao_ui_core::{
    HfButton, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell, HfTableHeader,
    HfTableHeaderCell, HfTableRow, HfTextarea,
};
use leptos::prelude::*;

const DEFAULT_SCRIPT: &str = "// Simple Moving Average\nplot(sma(close, 20))";

const MOCK_DIAGNOSTICS: [&str; 2] = [
    "Line 2: 'sma' resolved to built-in indicator",
    "0 errors, 0 warnings",
];

/// Static mock catalogue of built-in indicators.
const INDICATOR_LIBRARY: [(&str, &str); 8] = [
    ("SMA", "Simple moving average of closing prices over N bars"),
    (
        "EMA",
        "Exponential moving average with smoothing factor 2 / (N + 1)",
    ),
    (
        "RSI",
        "Relative strength index, momentum oscillator on a 0-100 scale",
    ),
    (
        "MACD",
        "Difference between fast and slow EMAs with a signal line",
    ),
    (
        "Bollinger Bands",
        "Moving average plus and minus K standard deviations",
    ),
    (
        "ATR",
        "Average true range, a volatility measure over N bars",
    ),
    (
        "VWAP",
        "Volume-weighted average price since the session start",
    ),
    (
        "Stochastic",
        "Position of the close within the recent high-low range",
    ),
];

#[component]
pub fn OpenScriptPanel() -> impl IntoView {
    let diagnostics = RwSignal::new(Vec::<String>::new());

    let on_compile = Callback::new(move |_: web_sys::MouseEvent| {
        diagnostics.set(
            MOCK_DIAGNOSTICS
                .iter()
                .map(|line| line.to_string())
                .collect(),
        );
    });

    view! {
        <HfPanel class="openscript">
            <HfPanelHeader title="OpenScript Editor" />
            // HfTextarea (ui-core) is the multi-line control, so no raw
            // <textarea> is needed. The mock editor is uncontrolled: the
            // script text is never compiled, so it is not tracked.
            <HfTextarea
                class="openscript__editor"
                rows=8
                value=DEFAULT_SCRIPT.to_string()
            />
            <div class="openscript__actions">
                <HfButton on_click=on_compile>"Compile"</HfButton>
            </div>
            <ul class="openscript__diagnostics">
                {move || {
                    diagnostics
                        .get()
                        .into_iter()
                        .map(|line| view! { <li>{line}</li> })
                        .collect_view()
                }}
            </ul>
            <HfPanelHeader title="Indicator Library" />
            <HfTable class="openscript__library">
                <HfTableHeader>
                    <HfTableRow>
                        <HfTableHeaderCell>"Name"</HfTableHeaderCell>
                        <HfTableHeaderCell>"Description"</HfTableHeaderCell>
                    </HfTableRow>
                </HfTableHeader>
                <HfTableBody>
                    {INDICATOR_LIBRARY
                        .into_iter()
                        .map(|(name, description)| {
                            view! {
                                <HfTableRow>
                                    <HfTableCell>{name}</HfTableCell>
                                    <HfTableCell>{description}</HfTableCell>
                                </HfTableRow>
                            }
                        })
                        .collect_view()}
                </HfTableBody>
            </HfTable>
        </HfPanel>
    }
}
