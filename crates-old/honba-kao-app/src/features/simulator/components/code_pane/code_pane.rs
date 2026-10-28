//! The code pane — a minimal in-browser code editor for strategy graph source.
//! Replaces Monaco/CodeMirror with a lightweight regex-highlighted textarea,
//! keeping the CSR bundle size minimal (no external editor dependency).
//! Ported from fe's `code_pane` / `ai-optimizer-engine.tsx` patterns.

use leptos::prelude::*;

/// Language modes for the code gutter.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CodeMode {
    StrategySource, // strategy graph source (.grams)
    Python,         // Python strategy source
    Rust,           // Rust strategy source
}

/// Syntax-highlighted span: (start offset, end offset, CSS class name).
type HighlightSpan = (usize, usize, &'static str);

/// Highlight code using simple regex-based rules (no external parser).
/// Returns a vector of `(start, end, class)` tuples over the source code lines.
fn highlight_strategy_source(code: &str) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let lines: Vec<&str> = code.lines().collect();
    for (line_idx, line) in lines.iter().enumerate() {
        let line_offset = code[..code.as_bytes().iter().take(line_idx * 80 + line.as_bytes().len()).iter().take(code.as_bytes().len()).count()];
        // Simple keyword highlighting
        for (keyword, cls) in &[
            ("universe", "kw-universe"),
            ("strategy", "kw-strategy"),
            ("indicator", "kw-indicator"),
            ("condition", "kw-condition"),
            ("transform", "kw-transform"),
            ("sizing", "kw-sizing"),
            ("risk", "kw-risk"),
            ("order", "kw-order"),
            ("schedule", "kw-schedule"),
            ("benchmark", "kw-benchmark"),
        ] {
            if let Some(pos) = line.find(keyword) {
                spans.push((pos, pos + keyword.len(), cls));
            }
        }
        // Highlight numbers (simplified)
        if let Some(pos) = line.find(|c: char| c.is_ascii_digit()) {
            // naive: find run of digits
            let mut end = pos;
            while end < line.len() && line.as_bytes()[end].is_ascii_digit() {
                end += 1;
            }
            spans.push((pos, end, "kw-number"));
        }
        // Highlight strings (simplified)
        if let Some(start) = line.find('"') {
            let mut end = start + 1;
            while end < line.len() && line.as_bytes()[end] != b'"' {
                end += 1;
            }
            if end > start {
                spans.push((start, end, "kw-string"));
            }
        }
        // Highlight comments
        if let Some(comment_start) = line.find("//") {
            spans.push((comment_start, line.len(), "kw-comment"));
        }
    }
    spans
}

/// The code pane textarea with line numbers and regex-based highlighting.
#[component]
pub fn CodePane {
    /// The strategy source code.
    #[prop(into)]
    code: Signal<String>,
    /// The language mode (affects gutter label and highlight rules).
    #[prop(into)]
    mode: CodeMode,
    /// Callback when the user types (debounced in the parent).
    #[prop(into)]
    on_change: Callback<String>,
    /// Show line numbers (default true).
    #[prop(into, optional)]
    show_line_numbers: Option<bool>,
}

let mode = mode;
let code = code;
let on_change = on_change;
let show_line_numbers = show_line_numbers.unwrap_or(true);

let line_numbers = show_line_numbers.then(|| {
    (1..=((*code.get()).lines().count().max(1)).max(1)).map(|i| i.to_string()).collect::<Vec<_>>()
}).unwrap_or_default();

let highlighted = highlight_strategy_source(&code);

let line_spans: Vec<(usize, String)> = highlighted
    .iter()
    .filter_map(|(start, end, cls)| {
        // Map global character offset to line-relative offset
        let line_start = (*code.get()).lines().take_while(|l| code.as_bytes().iter().take(code.as_bytes().iter().take_while(|&c| *c != b'\n' as u8).count()).count()).count();
        // Simplified: just return the span data for the parent to render
        Some((*start, cls.to_string()))
    })
    .collect();

let code_text = *code.get();

/// Line number rendering (simple list).
let line_numbers_html = if show_line_numbers {
    line_numbers
        .iter()
        .enumerate()
        .map(|(i, n)| {
            view! { <div class="code-line-num">{n}</div> }.into_any()
        })
        .collect::<IntoView>()
} else {
    view! {}.into_any()
};

let highlighted_spans_html: Vec<web_sys::HtmlElement> = highlighted
    .iter()
    .map(|(start, end, cls)| {
        // Extract the substring for this span
        let code_clone = code_text.clone();
        let substring = &code_clone[*start..*end.min(code_clone.len())];
        view! { <span class=cls>{substring}</span> }.into_any()
    })
    .collect();

view! {
    <div class="code-pane">
      <div class="code-pane__gutter">{line_numbers_html}</div>
      <div class="code-pane__content">
        <textarea
          class="code-pane__textarea"
          rows="20"
          on:input=move |ev| {
              let new_code = ev.target().unwrap().dyn_into::<web_sys::HtmlTextAreaElement>().unwrap().value();
              on_change.call(new_code);
          }
          value=code_text
          style="width: 100%; height: 100%; font-family: var(--mono-font, "Courier New", monospace); font-size: 13px; line-height: 1.5; border: 1px solid var(--hair); border-radius: 4px; padding: 8px; resize: vertical;"
        />
      </div>
      <div class="code-pane__highlight">
        {move || {
            let spans = highlighted();
            highlighted_spans_html.iter().map(|el| el.clone()).collect::<IntoView>()
        }}
      </div>
    </div>
}
