//! `HfAiArtifactCard` — interactive card rendering AI-generated structured
//! artifacts (valuation matrices, thesis summaries, scenario tables). OpenTerminalUI
//! `AiVisuals` equivalent.
//!
//! The `content` prop is rendered as lightweight Markdown (headers, bold,
//! italic, inline code, bullet lists) so AI research artifacts display with
//! structure instead of a flat pre-wrapped string. nsh has no raw-HTML
//! injection, so the Markdown is parsed into [`MdBlock`]s and [`MdSpan`]s
//! and built as a real element tree.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// One block-level Markdown element.
#[derive(Debug, Clone, PartialEq)]
enum MdBlock {
    /// Level 1-6, raw inline text (emphasis markers still inside).
    Heading(u8, String),
    Paragraph(String),
    ListItem(String),
}

/// A run of inline text sharing one emphasis style.
#[derive(Debug, Clone, PartialEq)]
struct MdSpan {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
}

impl MdSpan {
    fn plain(text: String) -> Self {
        Self {
            text,
            bold: false,
            italic: false,
            code: false,
        }
    }
}

/// Index of the first occurrence of `pat` in `chars` at or after `from`.
fn find_delim(chars: &[char], from: usize, pat: &[char]) -> Option<usize> {
    if chars.len() < pat.len() || from > chars.len() - pat.len() {
        return None;
    }
    (from..=chars.len() - pat.len()).find(|&j| chars[j..j + pat.len()] == *pat)
}

fn flush_plain(plain: &mut String, spans: &mut Vec<MdSpan>) {
    if !plain.is_empty() {
        spans.push(MdSpan::plain(std::mem::take(plain)));
    }
}

/// Split inline Markdown emphasis (`**bold**`, `*italic*`, `` `code` ``) into
/// spans. An unclosed marker is kept as literal text.
fn parse_inline(s: &str) -> Vec<MdSpan> {
    let chars: Vec<char> = s.chars().collect();
    let mut spans = Vec::new();
    let mut plain = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '*' && chars.get(i + 1) == Some(&'*') {
            if let Some(j) = find_delim(&chars, i + 2, &['*', '*']) {
                flush_plain(&mut plain, &mut spans);
                spans.push(MdSpan {
                    text: chars[i + 2..j].iter().collect(),
                    bold: true,
                    italic: false,
                    code: false,
                });
                i = j + 2;
                continue;
            }
        } else if chars[i] == '*' {
            if let Some(j) = find_delim(&chars, i + 1, &['*']) {
                flush_plain(&mut plain, &mut spans);
                spans.push(MdSpan {
                    text: chars[i + 1..j].iter().collect(),
                    bold: false,
                    italic: true,
                    code: false,
                });
                i = j + 1;
                continue;
            }
        } else if chars[i] == '`' {
            if let Some(j) = find_delim(&chars, i + 1, &['`']) {
                flush_plain(&mut plain, &mut spans);
                spans.push(MdSpan {
                    text: chars[i + 1..j].iter().collect(),
                    bold: false,
                    italic: false,
                    code: true,
                });
                i = j + 1;
                continue;
            }
        }
        plain.push(chars[i]);
        i += 1;
    }
    flush_plain(&mut plain, &mut spans);
    spans
}

/// Split content into block-level Markdown elements (headers, bullet items,
/// non-blank paragraph lines).
fn parse_markdown(content: &str) -> Vec<MdBlock> {
    let mut blocks = Vec::new();
    for line in content.split('\n') {
        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&level) && trimmed.chars().nth(level) == Some(' ') {
            let text = trimmed[level..].trim_start();
            blocks.push(MdBlock::Heading(level as u8, text.to_string()));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- ") {
            blocks.push(MdBlock::ListItem(rest.to_string()));
            continue;
        }
        if !trimmed.is_empty() {
            blocks.push(MdBlock::Paragraph(trimmed.to_string()));
        }
    }
    blocks
}

fn heading_size(level: u8) -> f32 {
    match level {
        1 => 16.0,
        2 => 14.0,
        3 => 13.0,
        _ => 12.0,
    }
}

/// One line of inline spans, each span its own styled `text`.
fn inline_row(spans: Vec<MdSpan>, size: f32, weight: FontWeight) -> View {
    let mut row = view().flex_row().flex_wrap().items_center();
    for span in spans {
        let w = if span.bold { FontWeight::BOLD } else { weight };
        let mut t = text(span.text)
            .text_size(px(size))
            .font_weight(w)
            .text_high();
        if span.italic {
            t = t.italic();
        }
        if span.code {
            t = t
                .font_family("monospace")
                .px(px(3.0))
                .rounded(px(3.0))
                .surface_muted();
        }
        row = row.child(t);
    }
    row
}

/// A bullet item: a "•" glyph beside its inline spans.
fn list_item(raw: &str) -> View {
    view()
        .flex_row()
        .items_start()
        .gap(px(6.0))
        .child(text("•").text_size(px(12.0)).text_muted())
        .child(inline_row(parse_inline(raw), 12.0, FontWeight::NORMAL))
}

/// Build the element tree for the parsed blocks. Consecutive list items are
/// grouped under one column (nsh has no native `<ul>`).
fn render_blocks(blocks: Vec<MdBlock>) -> View {
    let mut body = view().flex_col().gap(px(6.0));
    let mut iter = blocks.into_iter().peekable();
    while let Some(block) = iter.next() {
        body = match block {
            MdBlock::Heading(level, raw) => body.child(inline_row(
                parse_inline(&raw),
                heading_size(level),
                FontWeight::BOLD,
            )),
            MdBlock::Paragraph(raw) => {
                body.child(inline_row(parse_inline(&raw), 12.0, FontWeight::NORMAL))
            }
            MdBlock::ListItem(first) => {
                let mut items = view()
                    .flex_col()
                    .gap(px(2.0))
                    .px(px(4.0))
                    .child(list_item(&first));
                while matches!(iter.peek(), Some(MdBlock::ListItem(_))) {
                    if let Some(MdBlock::ListItem(raw)) = iter.next() {
                        items = items.child(list_item(&raw));
                    }
                }
                body.child(items)
            }
        };
    }
    body
}

/// Props for [`HfAiArtifactCard`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfAiArtifactCardProps {
    pub title: Prop<String>,
    /// Parsed once, at construction (static display, like the other
    /// molecules of this port).
    pub content: Prop<String>,
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// A bordered card with a title row, an "AI" badge, and the Markdown-rendered
/// artifact body.
#[component]
pub fn HfAiArtifactCard(props: HfAiArtifactCardProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let title = props.title;
    let extra = props.class.get_untracked().unwrap_or_default();
    let blocks = render_blocks(parse_markdown(&props.content.get_untracked()));

    let header = view()
        .flex_row()
        .items_center()
        .justify_between()
        .px(px(12.0))
        .py(px(8.0))
        .border_b_1()
        .border_soft()
        .child(
            text(move || title.get())
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_high(),
        )
        .child(
            view()
                .px(px(6.0))
                .py(px(1.0))
                .rounded(px(4.0))
                .surface_muted()
                .child(text("AI").text_size(px(10.0)).text_muted()),
        );

    view()
        .flex_col()
        .rounded(px(8.0))
        .border(px(1.0))
        .border_soft()
        .surface_base()
        .app_class(Some(SharedString::from(format!(
            "hf-ai-artifact-card {}",
            extra
        ))))
        .child(header)
        .child(view().px(px(12.0)).py(px(12.0)).child(blocks))
}
