//! `HfAiArtifactCard` — interactive card rendering AI-generated structured
//! artifacts (valuation matrices, thesis summaries, scenario tables). OpenTerminalUI
//! `AiVisuals` equivalent.
//!
//! The `content` prop is rendered as lightweight Markdown (headers, bold,
//! italic, inline code, bullet lists) so AI research artifacts display with
//! structure instead of a flat pre-wrapped string.

use leptos::prelude::*;

/// Escape HTML special characters so literal text never breaks out of a tag.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render inline Markdown emphasis: `**bold**`, `*italic*`, `` `code` ``.
fn render_inline(s: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'*' {
            let mut j = i + 2;
            while j + 1 < bytes.len() && !(bytes[j] == b'*' && bytes[j + 1] == b'*') {
                j += 1;
            }
            if j + 1 < bytes.len() {
                out.push_str("<strong>");
                out.push_str(&escape_html(&s[i + 2..j]));
                out.push_str("</strong>");
                i = j + 2;
                continue;
            }
        } else if bytes[i] == b'*' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'*' {
                j += 1;
            }
            if j < bytes.len() {
                out.push_str("<em>");
                out.push_str(&escape_html(&s[i + 1..j]));
                out.push_str("</em>");
                i = j + 1;
                continue;
            }
        } else if bytes[i] == b'`' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'`' {
                j += 1;
            }
            if j < bytes.len() {
                out.push_str("<code>");
                out.push_str(&escape_html(&s[i + 1..j]));
                out.push_str("</code>");
                i = j + 1;
                continue;
            }
        }
        out.push_str(&escape_html(&s[i..i + 1]));
        i += 1;
    }
    out
}

/// Render lightweight Markdown (headers, bullet lists, paragraphs, inline
/// emphasis) into HTML.
fn render_markdown(content: &str) -> String {
    let mut out = String::new();
    let mut in_list = false;
    for line in content.split('\n') {
        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|c| *c == '#').count();
        if level >= 1 && level <= 6 && trimmed.chars().nth(level) == Some(' ') {
            if in_list {
                out.push_str("</ul>");
                in_list = false;
            }
            let text = trimmed[level..].trim_start();
            out.push_str(&format!("<h{}>{}</h{}>", level, render_inline(text), level));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- ") {
            if !in_list {
                out.push_str("<ul>");
                in_list = true;
            }
            out.push_str(&format!("<li>{}</li>", render_inline(rest)));
            continue;
        }
        if in_list {
            out.push_str("</ul>");
            in_list = false;
        }
        if !trimmed.is_empty() {
            out.push_str(&format!("<p>{}</p>", render_inline(trimmed)));
        }
    }
    if in_list {
        out.push_str("</ul>");
    }
    out
}

#[component]
pub fn HfAiArtifactCard(
    #[prop(into)] title: String,
    #[prop(into)] content: String,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let base = "hf-ai-artifact-card rounded-lg border border-border-soft bg-surface-base shadow-sm flex flex-col";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    let rendered = render_markdown(&content);
    view! {
        <div class=classes>
            <div class="hf-artifact-header px-3 py-2 border-b border-border-soft flex items-center justify-between">
                <span class="text-xs font-semibold text-text-high">{title}</span>
                <span class="hf-kbd-badge text-[10px] bg-surface-subtle text-text-muted">"AI"</span>
            </div>
            <div class="hf-artifact-body p-3 break-words text-xs text-text-high"
                inner_html=rendered
            />
        </div>
    }
}
