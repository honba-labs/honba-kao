//! `HfGoBar` — fast `Ctrl+G` modal search input with symbol and route
//! auto-complete. Ported from the restructure spec (Section 7.6) and
//! OpenTerminalUI `CommandBar`.
//!
//! Built directly on [`nsh_base::Sheet`] (not `HfDialog`/`HfModal`, which
//! add a title bar / confirm-cancel footer this bar doesn't want) — `Sheet`
//! already handles backdrop-press-to-close and Escape-to-close internally,
//! so this file never hand-rolls a backdrop click guard or an Escape key
//! binding; it only listens for `SheetEvent::OpenChange(false, _)`, the one
//! authoritative "please close" signal for every real dismissal.
//!
//! `open` is caller-owned, same convention as every other molecule in this
//! port: the component never closes itself, it only reports that a close
//! was requested (via `Sheet`) or that a destination was chosen.
//!
//! Local widget state (`query`, `highlighted`) is pure transient UI state,
//! not business state the caller needs to own — it lives entirely inside
//! this component via `signal`/`computed`/`effect`, same as
//! `agent_console.rs`'s `draft`.
//!
//! Arrow-key navigation is bound via [`on_key_with`] with
//! `When::Always`: a plain `ArrowUp`/`ArrowDown` is classified by nsh's
//! keymap as "edits text", so with the default `When::Auto` it would not
//! fire while the input has focus. The input itself only exposes
//! `on_change`/`on_submit` (via the raw [`nsh_core::element::input`]
//! builder, same reason `agent_console.rs` uses it instead of `HfInput`).
//!
//! Dropped from the original Leptos version: scroll-the-highlighted-row-
//! into-view on arrow-key navigation. nsh has no "scroll this child into
//! view" primitive (`ElementRef::scroll_to` only scrolls to an absolute
//! offset, not a child's position) — same kind of gap already documented
//! elsewhere in this port (e.g. `collateral_asset_pane`'s dropped
//! max-height scroll note).

use nsh_base::{Sheet, SheetEvent, SheetProps};
use nsh_core::element::input;
use nsh_core::element::{Style, StyleRefinement};
use nsh_core::prelude::*;

use crate::primitive::kbd::{HfKbdBadge, HfKbdBadgeProps};
use crate::primitive::prompt::{HfPromptSymbol, HfPromptSymbolProps};
use crate::theme::tokens::HonbaStyle;

/// The events of [`HfGoBar`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfGoBarEvent {
    /// Close requested (Escape or a backdrop press via `Sheet`, or after a
    /// selection/submit below): the caller sets `open` to `false`.
    Close,
    /// A destination was chosen (a suggestion clicked/Enter-selected, or
    /// the typed query submitted with no match): the caller navigates and
    /// should also treat this as implying [`HfGoBarEvent::Close`].
    Navigate(String),
}

/// Props for [`HfGoBar`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfGoBarProps {
    /// Open. The app owns the value, same convention as `HfDialog`'s
    /// `open` (`primitive/base/dialog.rs`).
    pub open: Prop<bool>,
    /// Symbols/commands the live substring filter matches against.
    #[prop(default)]
    pub autocomplete: Prop<Vec<String>>,
}

/// The panel's surface, border and size, mirroring the original Leptos
/// markup's `w-[600px] max-w-full rounded-lg bg-surface-base border
/// border-border-strong shadow-2xl p-3`.
fn panel_style() -> StyleRefinement {
    StyleRefinement::default()
        .surface_base()
        .border_strong()
        .rounded(px(8.0))
        .w(px(600.0))
        .p(px(12.0))
        .shadow_2xl()
}

/// A translucent backdrop behind the panel, same tone every `Sheet`
/// wrapper in this port uses (`primitive/base/dialog.rs`).
fn backdrop_style() -> StyleRefinement {
    StyleRefinement::default().bg(AdaptiveColor::rgb(0x000000, 0x000000).opacity(0.6))
}

/// `Ctrl+G` modal command palette: a text input with live substring-
/// filtered autocomplete, arrow-key highlight (clamped, not wraparound —
/// matches the original's actual behavior), Enter to select/submit,
/// Escape/backdrop to close (via `Sheet`).
#[component]
pub fn HfGoBar(props: HfGoBarProps, cx: Scope<HfGoBarEvent>) -> impl IntoElement {
    let is_open = props.open;
    let autocomplete = props.autocomplete;

    let query = signal(String::new());
    let highlighted = signal(0_usize);

    // Reset query/highlight whenever the bar opens (mirrors the original's
    // `Effect::new` that reset `query`/`highlighted`/focus on `open`).
    effect(move || {
        if is_open.get() {
            query.set(String::new());
            highlighted.set(0);
        }
    });

    let filtered = computed(move || {
        let q = query.get().trim().to_uppercase();
        if q.is_empty() {
            Vec::<String>::new()
        } else {
            autocomplete
                .get()
                .iter()
                .filter(|s| s.to_uppercase().contains(&q))
                .cloned()
                .collect::<Vec<_>>()
        }
    });

    // Keep the highlight inside the visible suggestion list.
    effect(move || {
        let n = filtered.get().len();
        if n == 0 {
            highlighted.set(0);
        } else if highlighted.get() >= n {
            highlighted.set(n.saturating_sub(1));
        }
    });

    // Ambient key bindings: must fire even while the input has focus
    // (arrow keys otherwise "edit text" under the default `When::Auto`).
    // Harmless no-ops while the bar is closed, same as the original's
    // always-attached `document:keydown` listener (gated only incidentally
    // there, via the subtree being conditionally shown).
    on_key_with(
        "up",
        KeyOptions {
            when: nsh_core::keymap::When::Always,
            repeat: false,
        },
        move |_| {
            let n = filtered.get().len();
            highlighted.set(highlighted.get().saturating_sub(1).min(n.saturating_sub(1)));
        },
    );
    on_key_with(
        "down",
        KeyOptions {
            when: nsh_core::keymap::When::Always,
            repeat: false,
        },
        move |_| {
            let n = filtered.get().len();
            highlighted.set(
                (highlighted.get() + 1)
                    .max(1)
                    .min(n.saturating_sub(1).max(1)),
            );
        },
    );

    let submit_cx = cx.clone();
    let field = input()
        .value(move || query.get())
        .placeholder(
            "GO: Enter symbol (e.g. INFY, BTC), command (/dom, /fno), or action...".to_string(),
        )
        .flex_1()
        .font_family("monospace")
        .text_size(px(13.0))
        .text_high()
        .auto_focus(true)
        .on_change(move |s: &str| query.set(s.to_string()))
        .on_submit(move |_: &str| {
            let list = filtered.get();
            let sel = highlighted.get();
            if !list.is_empty() && sel < list.len() {
                let choice = list[sel].clone();
                submit_cx.emit(HfGoBarEvent::Navigate(choice));
                submit_cx.emit(HfGoBarEvent::Close);
            } else {
                let q = query.get().trim().to_uppercase();
                if !q.is_empty() {
                    submit_cx.emit(HfGoBarEvent::Navigate(q));
                    submit_cx.emit(HfGoBarEvent::Close);
                }
            }
        });

    let input_row = view()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .mb(px(8.0))
        .child(HfPromptSymbol::element(HfPromptSymbolProps {
            symbol: Prop::from(">_".to_string()),
        }))
        .child(field)
        .child(HfKbdBadge::element(HfKbdBadgeProps {
            key: Prop::from("ESC".to_string()),
        }));

    // Only present while there is something to show. Built via `dynamic`
    // (returning `Option<AnyElement>`) rather than the private `when`/`If`
    // flow helper (`nsh_core::element::flow` is `pub(crate)`, unreachable
    // from this crate; `Option<T: IntoElement>` implements `IntoElements`,
    // confirmed in `nsh-core-0.3.0/src/element/flow.rs`). Gated on a
    // `Memo<bool>` (not `filtered.get()` directly) so the whole panel +
    // `each()` list is rebuilt only on the empty/non-empty *transition* —
    // `computed` "notifies only when its result changed" (see its own doc
    // comment in `nsh-core-0.3.0/src/element/reactive.rs`) — while
    // `each()`'s own internal diffing (not this outer rebuild) handles
    // ordinary match-count changes once mounted.
    let has_matches = computed(move || !filtered.get().is_empty());
    let suggestions_cx = cx.clone();
    let suggestions = dynamic(move || -> Option<AnyElement> {
        if !has_matches.get() {
            return None;
        }
        let rows_cx = suggestions_cx.clone();
        let rows = each(
            move || filtered.get().into_iter().enumerate().collect::<Vec<_>>(),
            |(i, _): &(usize, String)| *i,
            move |entry: ReadSignal<(usize, String)>, _row: ReadSignal<usize>| {
                let cx = rows_cx.clone();
                view()
                    .px(px(12.0))
                    .py(px(6.0))
                    .text_size(px(12.0))
                    .border_b_1()
                    .border_soft()
                    .app_class(move || {
                        Some(SharedString::from(
                            if highlighted.get() == entry.get_untracked().0 {
                                "cursor-pointer bg-emerald-600/20 text-emerald-400 font-bold"
                            } else {
                                "cursor-pointer hover:bg-surface-subtle"
                            },
                        ))
                    })
                    .on_press(move |_: &()| {
                        let choice = entry.get_untracked().1.clone();
                        cx.emit(HfGoBarEvent::Navigate(choice));
                        cx.emit(HfGoBarEvent::Close);
                    })
                    .child(text(move || entry.get().1).text_high())
            },
        );
        Some(
            scroll()
                .h(px(200.0))
                .mb(px(8.0))
                .rounded(px(4.0))
                .border(px(1.0))
                .border_soft()
                .surface_base()
                .child(view().flex_col().child(rows))
                .into_element(),
        )
    });

    let hint = text(move || {
        let n = filtered.get().len();
        if n > 0 {
            format!(
                "\u{2191}/\u{2193} navigate \u{00b7} Enter select \u{00b7} {n} match{}",
                if n == 1 { "" } else { "es" }
            )
        } else {
            "\u{2191}/\u{2193} navigate \u{00b7} Enter submit".to_string()
        }
    })
    .text_size(px(10.0))
    .text_muted();

    let panel = view()
        .flex_col()
        .child(input_row)
        .child(suggestions)
        .child(hint);

    Sheet::element(SheetProps {
        open: Prop::bind(move || Some(is_open.get())),
        side: Prop::from(Placement::Top),
        close_on_escape: Prop::from(true),
        close_on_backdrop_press: Prop::from(true),
        popup_style: Prop::from(Style::from(panel_style())),
        backdrop_style: Prop::from(Style::from(backdrop_style())),
        ..Default::default()
    })
    .child(panel)
    .on_event(move |event: &SheetEvent| {
        if let SheetEvent::OpenChange(false, _) = event {
            cx.emit(HfGoBarEvent::Close);
        }
    })
}
