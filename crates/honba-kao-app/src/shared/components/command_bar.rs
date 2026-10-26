//! CommandBar: the persistent GO bar (`Ctrl+G`).
//!
//! Port of OpenTerminalUI `layout/CommandBar.tsx`, behaviour-first:
//! * `Ctrl+G` focuses and selects the input from anywhere;
//! * `Ctrl+R` in the input toggles reverse history search;
//! * typing shows ticker hints (`RELIANCE DES`, `INFY FA ...`), matching
//!   functions, mock symbol matches and fuzzy history matches;
//! * `ArrowUp`/`ArrowDown` move the highlight, `Enter` runs the highlighted
//!   suggestion (or the raw text), `Escape` closes;
//! * submitted commands go to a capped history under `ot:gobar:history:v1`
//!   (localStorage, best effort: every read and write is guarded).
//!
//! The preview card for an exact mock ticker shows LTP, change, and a sparkline.
//! It is MOCK data; see [`crate::shared::command::symbols`].

use honba_kao_ui_core::composed::command_bar::{
    CommandSuggestion, HfAssetPreviewCard, HfCommandBar, HfCommandBarInput, HfCommandSuggestions,
};
use honba_kao_ui_core::trading::{HfProvenanceChip, ProvenanceQuality};
use leptos::html;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::shared::command::commanding::{
    COMMAND_FUNCTIONS, CommandOutcome, MAX_HISTORY, commit_history, fuzzy_score, parse_command,
    route_for, ticker_hints,
};
use crate::shared::command::symbols::{self, MockSymbol};
use crate::shared::stream::feed::MarketFeed;

/// localStorage key, same name as OpenTerminalUI's gobar history.
pub const HISTORY_KEY: &str = "ot:gobar:history:v1";

/// Reads the persisted history. Storage can be blocked or absent (private
/// windows, previews); every step falls back to an empty list.
pub fn read_history() -> Vec<String> {
    let raw = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(HISTORY_KEY).ok().flatten());
    raw.and_then(|r| serde_json::from_str::<Vec<String>>(&r).ok())
        .unwrap_or_default()
        .into_iter()
        .take(MAX_HISTORY)
        .collect()
}

/// Persists the history. Failures are ignored: the bar works without storage.
pub fn write_history(history: &[String]) {
    let Ok(raw) = serde_json::to_string(history) else { return };
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(HISTORY_KEY, &raw);
    }
}

/// One candidate row: the visible suggestion plus the command it runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub view: CommandSuggestion,
    pub command: String,
}

/// Builds the candidate list for the current input. Pure apart from the
/// signals it is called with, so it is easy to reason about.
pub fn build_candidates(query: &str, history: &[String], reverse: bool) -> Vec<Candidate> {
    let q = query.trim();
    let history_row = |cmd: &String| Candidate {
        view: CommandSuggestion {
            key: format!("hist:{cmd}"),
            title: cmd.clone(),
            subtitle: "Recent command".into(),
            badge: "HIST",
        },
        command: cmd.clone(),
    };

    if reverse {
        return history
            .iter()
            .filter(|cmd| q.is_empty() || fuzzy_score(cmd, q) >= 0)
            .map(history_row)
            .collect();
    }

    if q.is_empty() {
        return history.iter().take(8).map(history_row).collect();
    }

    let mut out: Vec<Candidate> = Vec::new();

    out.extend(ticker_hints(q).into_iter().map(|hint| Candidate {
        view: CommandSuggestion {
            key: hint.key,
            title: hint.title,
            subtitle: hint.subtitle,
            badge: "FUNC",
        },
        command: hint.command,
    }));

    out.extend(symbols::search(q, 6).into_iter().map(|s: MockSymbol| Candidate {
        view: CommandSuggestion {
            key: format!("asset:{}", s.ticker),
            title: s.ticker.to_string(),
            subtitle: format!("{} · {}", s.name, s.exchange),
            badge: "ASSET",
        },
        command: s.ticker.to_string(),
    }));

    out.extend(
        COMMAND_FUNCTIONS
            .iter()
            .filter(|spec| {
                fuzzy_score(spec.code, q) >= 0
                    || fuzzy_score(spec.label, q) >= 0
                    || spec.aliases.iter().any(|a| fuzzy_score(a, q) >= 0)
            })
            .take(6)
            .map(|spec| Candidate {
                view: CommandSuggestion {
                    key: format!("fn:{}", spec.code),
                    title: spec.code.to_string(),
                    subtitle: spec.description.to_string(),
                    badge: "CMD",
                },
                command: spec.code.to_string(),
            }),
    );

    out.extend(
        history
            .iter()
            .filter(|cmd| fuzzy_score(cmd, q) >= 0)
            .take(4)
            .map(history_row),
    );

    out
}

/// The mock ticker for the preview card: the first token, if it is a known symbol.
fn preview_symbol(query: &str) -> Option<MockSymbol> {
    let first = query.split_whitespace().next()?;
    symbols::lookup(first)
}

#[component]
pub fn CommandBar() -> impl IntoView {
    let navigate = use_navigate();
    let feed = use_context::<MarketFeed>();
    let value = RwSignal::new(String::new());
    let input_ref = NodeRef::<html::Input>::new();
    let is_open = RwSignal::new(false);
    let reverse = RwSignal::new(false);
    let selected = RwSignal::new(0usize);
    let moved = RwSignal::new(false);
    let history = RwSignal::new(read_history());
    let feedback = RwSignal::new(None::<String>);

    let candidates = Memo::new(move |_| {
        build_candidates(&value.get(), &history.get(), reverse.get())
    });
    let views = Signal::derive(move || candidates.get().into_iter().map(|c| c.view).collect::<Vec<_>>());
    let preview = Memo::new(move |_| preview_symbol(&value.get()));
    let heading = Signal::derive(move || {
        if reverse.get() {
            "Reverse History Search (Ctrl+R)".to_string()
        } else {
            "Suggestions".to_string()
        }
    });

    // Reset the highlight whenever the candidate list changes shape.
    Effect::new(move |_| {
        let _ = candidates.get();
        selected.set(0);
        moved.set(false);
    });

    let run = Callback::new(move |command: String| {
        let parsed = parse_command(&command);
        match route_for(&parsed) {
            CommandOutcome::Navigate(route) => {
                history.update(|h| commit_history(h, &command));
                write_history(&history.get_untracked());
                value.set(String::new());
                feedback.set(None);
                is_open.set(false);
                reverse.set(false);
                navigate(&route, NavigateOptions::default());
            }
            CommandOutcome::Unsupported(message) => {
                feedback.set(Some(message));
                is_open.set(true);
            }
        }
    });

    let submit_selected_or_raw = move || {
        let list = candidates.get_untracked();
        let index = selected.get_untracked();
        let raw = value.get_untracked();
        let use_selected = moved.get_untracked() || raw.trim().is_empty();
        if use_selected {
            if let Some(candidate) = list.get(index) {
                run.run(candidate.command.clone());
                return;
            }
        }
        if !raw.trim().is_empty() {
            run.run(raw);
        }
    };

    let focus_bar = move || {
        if let Some(el) = input_ref.get_untracked() {
            el.select();
            let _ = el.focus();
        }
        is_open.set(true);
        reverse.set(false);
    };

    // Global `Ctrl+G` (focus) and `Escape` (close) handled at window level.
    let handle = window_event_listener(leptos::ev::keydown, move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let mod_key = ev.ctrl_key() || ev.meta_key();
        if mod_key && key.eq_ignore_ascii_case("g") {
            ev.prevent_default();
            focus_bar();
        } else if key == "Escape" && is_open.get_untracked() {
            is_open.set(false);
            reverse.set(false);
            if let Some(el) = input_ref.get_untracked() {
                let _ = el.blur();
            }
        }
    });
    on_cleanup(move || handle.remove());

    let on_keydown = Callback::new(move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let mod_key = ev.ctrl_key() || ev.meta_key();
        if mod_key && key.eq_ignore_ascii_case("r") {
            ev.prevent_default();
            reverse.update(|r| *r = !*r);
            is_open.set(true);
            return;
        }
        match key.as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                let len = candidates.get_untracked().len();
                if len > 0 {
                    selected.update(|i| *i = (*i + 1).min(len - 1));
                    moved.set(true);
                }
                is_open.set(true);
            }
            "ArrowUp" => {
                ev.prevent_default();
                selected.update(|i| *i = i.saturating_sub(1));
                moved.set(true);
            }
            "Enter" => {
                ev.prevent_default();
                submit_selected_or_raw();
            }
            _ => {
                is_open.set(true);
                feedback.set(None);
            }
        }
    });

    let on_pick = Callback::new(move |idx: usize| {
        if let Some(candidate) = candidates.get_untracked().get(idx) {
            run.run(candidate.command.clone());
        }
    });

    view! {
        <HfCommandBar class="app-command-bar" open=Signal::derive(move || is_open.get())>
            <span class="hf-command-bar__prompt">"GO"</span>
            <HfCommandBarInput
                value=value
                node_ref=input_ref
                placeholder="Type ticker, command, or search...  (Ctrl+G)"
                on_keydown=on_keydown
                on_focus=Callback::new(move |_| is_open.set(true))
                on_blur=Callback::new(move |_| is_open.set(false))
            />
            {move || {
                feed.map(|f| {
                    let quality = f.quality().get();
                    view! { <HfProvenanceChip quality=quality compact=true title="Market feed provenance" /> }
                })
            }}
            <span class="hf-command-bar__hint">"Ctrl+K palette"</span>
            <Show when=move || is_open.get()>
                <div class="app-command-bar__dropdown">
                    <HfCommandSuggestions
                        items=views
                        selected=Signal::derive(move || selected.get())
                        on_pick=on_pick
                        heading=heading
                    />
                    {move || {
                        preview
                            .get()
                            .map(|s| {
                                // Live quote when the feed has one; otherwise the mock table row.
                                let live = feed.and_then(|f| f.snapshot(s.ticker));
                                let (ltp, change, quality) = match live {
                                    Some(q) => (q.ltp, q.change_pct, q.quality),
                                    None => (s.ltp.to_string(), s.change_pct.to_string(), ProvenanceQuality::Synthetic),
                                };
                                view! {
                                    <HfAssetPreviewCard
                                        ticker=s.ticker.to_string()
                                        name=s.name.to_string()
                                        exchange=s.exchange.to_string()
                                        asset_class=s.asset_class.to_string()
                                        ltp=ltp
                                        change_pct=change
                                        series=s.series.to_vec()
                                        quality=quality
                                    />
                                }
                            })
                    }}
                    {move || feedback.get().map(|msg| view! { <div class="app-command-bar__feedback">{msg}</div> })}
                </div>
            </Show>
        </HfCommandBar>
    }
}
