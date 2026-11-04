//! Screener filter bar (`screener-design-doc.md` §5.1).
//!
//! A plain reactive builder fn (the repo's `hf_tool_card` pattern — no
//! `#[derive(Props)]`): the bar mutates its page-owned signal params
//! directly and toggles the two modal-open signals.

use honba_kao_ui_core::molecules::quant::filter_pill::{
    HfFilterPill, HfFilterPillEvent, HfFilterPillProps,
};
use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::primitive::base::input::{HfInput, HfInputEvent, HfInputProps};
use honba_kao_ui_core::theme::tokens::{HonbaStyle, colors, radius};
use nsh_core::prelude::*;

use crate::features::screener::export_csv::deliver_csv;
use crate::features::screener::screener_heatmap::ScreenerViewMode;
use crate::features::screener::table_grid::export_records_csv;
use crate::features::screener::types::{
    FilterCriteria, MarketCapTier, QuickPreset, ScreenerAssetType, ScreenerRecord, ScreenerViewTab,
};

/// Exchange region selected via the screener's market-region flyout.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum MarketRegion {
    #[default]
    India,
    UnitedStates,
    UnitedKingdom,
}

impl MarketRegion {
    /// Every region, in flyout display order.
    pub const ALL: [MarketRegion; 3] = [
        MarketRegion::India,
        MarketRegion::UnitedStates,
        MarketRegion::UnitedKingdom,
    ];

    /// Human-readable label: flag + venue parenthetical.
    pub fn label(&self) -> &'static str {
        match self {
            MarketRegion::India => "🇮🇳 India (NSE / BSE)",
            MarketRegion::UnitedStates => "🇺🇸 United States (NYSE / NASDAQ)",
            MarketRegion::UnitedKingdom => "🇬🇧 United Kingdom (LSE)",
        }
    }
}

#[allow(non_snake_case, clippy::too_many_arguments)]
pub fn ScreenerFilterBar(
    asset: Signal<ScreenerAssetType>,
    preset: Signal<QuickPreset>,
    tab: Signal<ScreenerViewTab>,
    search: Signal<String>,
    criteria: Signal<FilterCriteria>,
    matches: Signal<usize>,
    total: Signal<usize>,
    sectors: Signal<Vec<String>>,
    filter_modal_open: Signal<bool>,
    column_modal_open: Signal<bool>,
    screen_modal_open: Signal<bool>,
    rows: Signal<Vec<ScreenerRecord>>,
    view_mode: Signal<ScreenerViewMode>,
    group_by_sector: Signal<bool>,
    market_region: Signal<MarketRegion>,
    index_selection: Signal<Vec<String>>,
    shortlisted: Signal<Vec<String>>,
) -> View {
    // Local flyout / AI-prompt state (nsh `signal` constructor).
    let region_open: Signal<bool> = signal(false);
    let watchlist_open: Signal<bool> = signal(false);
    let index_open: Signal<bool> = signal(false);
    let ai_open: Signal<bool> = signal(false);
    let ai_draft: Signal<String> = signal(String::new());
    let ai_sent: Signal<usize> = signal(0usize);

    // Row 1 (left) — asset switcher segments.
    let asset_switcher = view()
        .flex_row()
        .gap(px(4.0))
        .border_1()
        .border_color(colors::BORDER)
        .rounded(px(radius::CONTROL))
        .bg(colors::SURFACE)
        .px(px(2.0))
        .py(px(2.0))
        .children(ScreenerAssetType::ALL.iter().copied().map(|candidate| {
            view()
                .px(px(10.0))
                .py(px(2.0))
                .rounded(px(radius::CONTROL))
                .bg(move || {
                    if asset.get() == candidate {
                        colors::ACCENT
                    } else {
                        colors::RAISED
                    }
                })
                .child(
                    text(candidate.label())
                        .text_size(px(10.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if asset.get() == candidate {
                                colors::ON_ACCENT
                            } else {
                                colors::TEXT
                            }
                        }),
                )
                .on_press(move || asset.set(candidate))
        }));

    // Row 1 (right) — quick-preset shelf with horizontal overflow.
    let preset_shelf = scroll()
        .flex_1()
        .flex_row()
        .gap(px(6.0))
        .px(px(4.0))
        .children(QuickPreset::ALL.iter().copied().map(|candidate| {
            HfFilterPill::element(HfFilterPillProps {
                label: Prop::from(candidate.label().to_string()),
                active: Prop::bind(move || preset.get() == candidate),
            })
            .on_event(move |event: &HfFilterPillEvent| {
                if *event == HfFilterPillEvent::Toggle {
                    preset.update(|current| {
                        *current = if *current == candidate {
                            QuickPreset::All
                        } else {
                            candidate
                        };
                    });
                }
            })
        }));

    let switcher_row = view()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .child(asset_switcher)
        .child(preset_shelf);

    // Row 2 — view-category tabs.
    let category_tabs = view()
        .flex_row()
        .gap(px(2.0))
        .border_1()
        .border_color(colors::BORDER)
        .rounded(px(radius::CONTROL))
        .bg(colors::SURFACE)
        .px(px(2.0))
        .py(px(2.0))
        .children(ScreenerViewTab::ALL.iter().copied().map(|candidate| {
            view()
                .px(px(10.0))
                .py(px(3.0))
                .rounded(px(radius::INSET))
                .bg(move || {
                    if tab.get() == candidate {
                        colors::RAISED
                    } else {
                        colors::SURFACE
                    }
                })
                .child(
                    text(candidate.label())
                        .text_size(px(10.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if tab.get() == candidate {
                                colors::TEXT
                            } else {
                                colors::MUTED
                            }
                        }),
                )
                .on_press(move || tab.set(candidate))
        }));

    // Row 3 (left) — symbol search + drill-down trigger pills.
    let search_field = view().w(px(240.0)).child(
        HfInput::element(HfInputProps {
            value: Prop::bind(move || search.get()),
            placeholder: Prop::from(Some("Search symbol, company, sector…".to_string())),
            disabled: Prop::from(false),
        })
        .on_event(move |event: &HfInputEvent| {
            let HfInputEvent::Change(value) = event;
            search.set(value.clone());
        }),
    );

    let drilldown_cluster = view()
        .flex_1()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(search_field)
        .child(
            HfButton::element(ghost_button_props())
                .child(text(move || {
                    format!("Sector: {}", sector_label(&criteria.get().sector))
                }))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        let roster = sectors.get();
                        if roster.is_empty() {
                            return;
                        }
                        let current = criteria.get().sector;
                        let current_idx = if current.is_empty() {
                            0
                        } else {
                            roster
                                .iter()
                                .position(|entry| entry == &current)
                                .unwrap_or(0)
                        };
                        let next = roster[(current_idx + 1) % roster.len()].clone();
                        criteria.update(|c| {
                            c.sector = if next == "All" { String::new() } else { next };
                        });
                    }
                }),
        )
        .child(
            HfButton::element(ghost_button_props())
                .child(text(move || {
                    format!("Cap: {}", criteria.get().market_cap_tier.label())
                }))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        let current = criteria.get().market_cap_tier;
                        let current_idx = MarketCapTier::ALL
                            .iter()
                            .position(|tier| *tier == current)
                            .unwrap_or(0);
                        let next_idx = (current_idx + 1) % MarketCapTier::ALL.len();
                        let next = MarketCapTier::ALL[next_idx];
                        criteria.update(|c| c.market_cap_tier = next);
                    }
                }),
        )
        .child(
            HfButton::element(ghost_button_props())
                .child(text(move || {
                    let active = criteria.get().active_count();
                    if active > 0 {
                        format!("Filters ({active})")
                    } else {
                        "Filters".to_string()
                    }
                }))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        filter_modal_open.set(true);
                    }
                }),
        );

    // Row 3 (right) — view switcher + grouping, match counter + reset/columns/export actions.
    let view_switcher = view()
        .flex_row()
        .gap(px(2.0))
        .border_1()
        .border_color(colors::BORDER)
        .rounded(px(radius::CONTROL))
        .bg(colors::SURFACE)
        .px(px(2.0))
        .py(px(2.0))
        .children(ScreenerViewMode::ALL.iter().copied().map(|mode| {
            view()
                .px(px(10.0))
                .py(px(2.0))
                .rounded(px(radius::CONTROL))
                .bg(move || {
                    if view_mode.get() == mode {
                        colors::RAISED
                    } else {
                        colors::SURFACE
                    }
                })
                .child(
                    text(mode.label())
                        .text_size(px(10.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if view_mode.get() == mode {
                                colors::TEXT
                            } else {
                                colors::MUTED
                            }
                        }),
                )
                .on_press(move || view_mode.set(mode))
        }));

    let grouping_toggle = HfButton::element(ghost_button_props())
        .child(text(move || {
            if group_by_sector.get() {
                "Group: Sector"
            } else {
                "Group: Flat"
            }
        }))
        .on_event(move |event: &HfButtonEvent| {
            if *event == HfButtonEvent::Press {
                group_by_sector.update(|grouped| *grouped = !*grouped);
            }
        });

    let actions_cluster = view()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .child(view_switcher)
        .child(grouping_toggle)
        .child(
            text(move || format!("Matches: {} / {}", matches.get(), total.get()))
                .text_muted()
                .text_size(px(10.5)),
        )
        .child(
            HfButton::element(ghost_button_props())
                .child(text("Reset"))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        criteria.set(FilterCriteria::default());
                    }
                }),
        )
        .child(
            HfButton::element(ghost_button_props())
                .child(text("Columns"))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        column_modal_open.set(true);
                    }
                }),
        )
        .child(
            HfButton::element(ghost_button_props())
                .child(text("Screens"))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        screen_modal_open.set(true);
                    }
                }),
        )
        .child(
            HfButton::element(ghost_button_props())
                .child(text("Export CSV"))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        let csv = export_records_csv(&rows.get());
                        tracing::info!(bytes = csv.len(), "screener export CSV");
                        deliver_csv("honba-screener.csv", &csv);
                    }
                }),
        );

    // Row 3 (middle) — contextual flyout triggers: region, watchlist, index, AI.
    let region_trigger = HfButton::element(ghost_button_props())
        .child(text(move || {
            format!("Region: {}", market_region.get().label())
        }))
        .on_event(move |event: &HfButtonEvent| {
            if *event == HfButtonEvent::Press {
                region_open.update(|open| *open = !*open);
            }
        });

    let watchlist_trigger = HfButton::element(ghost_button_props())
        .child(text(move || {
            format!("Watchlist ({})", shortlisted.get().len())
        }))
        .on_event(move |event: &HfButtonEvent| {
            if *event == HfButtonEvent::Press {
                watchlist_open.update(|open| *open = !*open);
            }
        });

    let index_trigger = HfButton::element(ghost_button_props())
        .child(text(move || {
            let count = index_selection.get().len();
            if count == 0 {
                "Index: All".to_string()
            } else {
                format!("Index: {count}")
            }
        }))
        .on_event(move |event: &HfButtonEvent| {
            if *event == HfButtonEvent::Press {
                index_open.update(|open| *open = !*open);
            }
        });

    // `✦ AI` pill: accent ink; toggles the prompt row under the filter bar.
    let ai_trigger = view()
        .flex_row()
        .items_center()
        .px(px(10.0))
        .py(px(3.0))
        .rounded(px(radius::CONTROL))
        .border_1()
        .border_color(colors::ACCENT)
        .bg(move || {
            if ai_open.get() {
                colors::ACCENT
            } else {
                colors::SURFACE
            }
        })
        .child(
            text(move || {
                let sent = ai_sent.get();
                if sent > 0 {
                    format!("✦ AI ({sent})")
                } else {
                    "✦ AI".to_string()
                }
            })
            .text_size(px(10.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(move || {
                if ai_open.get() {
                    colors::ON_ACCENT
                } else {
                    colors::ACCENT
                }
            }),
        )
        .on_press(move || ai_open.update(|open| *open = !*open));

    let context_cluster = view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(region_trigger)
        .child(watchlist_trigger)
        .child(index_trigger)
        .child(ai_trigger);

    let search_row = view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .w_full()
        .child(drilldown_cluster)
        .child(context_cluster)
        .child(actions_cluster);

    // Flat page bar: no card finish.
    view()
        .flex_col()
        .w_full()
        .gap(px(6.0))
        .px(px(8.0))
        .py(px(8.0))
        .bg(colors::BACKGROUND)
        .border_1()
        .border_color(colors::BORDER)
        .child(switcher_row)
        .child(category_tabs)
        .child(search_row)
        .child(when(region_open, move || {
            dynamic(move || market_region_flyout(market_region))
        }))
        .child(when(watchlist_open, move || {
            dynamic(move || watchlist_flyout(preset, total, shortlisted))
        }))
        .child(when(index_open, move || {
            dynamic(move || index_drawer(index_selection))
        }))
        .child(when(ai_open, move || {
            dynamic(move || ai_prompt_row(ai_open, ai_draft, ai_sent))
        }))
}

/// Shared chrome for the row-3 action buttons: small ghost buttons.
fn ghost_button_props() -> HfButtonProps {
    HfButtonProps {
        href: Prop::default(),
        size: Prop::from(Some(ButtonSize::Sm)),
        variant: Prop::from(Some(ButtonVariant::Ghost)),
        full_width: Prop::from(false),
        disabled: Prop::from(false),
        aria_label: Prop::from(None),
        class: Prop::default(),
        children: Children::default(),
    }
}

/// Compact display name for the currently filtered sector.
fn sector_label(sector: &str) -> String {
    if sector.is_empty() || sector == "All" {
        "All".into()
    } else {
        sector.to_string()
    }
}

/// Broad-market universe for the index drawer.
const BROAD_MARKET_INDEXES: [&str; 5] = [
    "NIFTY 50",
    "NIFTY NEXT 50",
    "NIFTY MIDCAP 100",
    "NIFTY SMALLCAP 100",
    "NIFTY 500",
];

/// Sectoral universe for the index drawer.
const SECTORAL_INDEXES: [&str; 6] = [
    "BANK NIFTY",
    "NIFTY IT",
    "NIFTY AUTO",
    "NIFTY PHARMA",
    "NIFTY METAL",
    "NIFTY FMCG",
];

/// Placeholder for the `✦ AI` prompt input.
const AI_PLACEHOLDER: &str =
    "Ask AI: e.g. 'Show me large cap tech stocks with low P/E and high dividend'";

/// Market-region flyout: single-select option row under the filter bar.
fn market_region_flyout(selected: Signal<MarketRegion>) -> View {
    view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .w_full()
        .px(px(8.0))
        .py(px(6.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::SURFACE)
        .border_1()
        .border_color(colors::BORDER)
        .children(MarketRegion::ALL.iter().copied().map(|candidate| {
            view()
                .px(px(10.0))
                .py(px(3.0))
                .rounded(px(radius::CONTROL))
                .bg(move || {
                    if selected.get() == candidate {
                        colors::ACCENT
                    } else {
                        colors::SURFACE
                    }
                })
                .child(
                    text(candidate.label())
                        .text_size(px(10.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if selected.get() == candidate {
                                colors::ON_ACCENT
                            } else {
                                colors::TEXT
                            }
                        }),
                )
                .on_press(move || selected.set(candidate))
        }))
}

/// Watchlist flyout: preset shortcuts plus the red clear action.
fn watchlist_flyout(
    preset: Signal<QuickPreset>,
    total: Signal<usize>,
    shortlisted: Signal<Vec<String>>,
) -> View {
    view()
        .flex_col()
        .gap(px(4.0))
        .w_full()
        .px(px(8.0))
        .py(px(6.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::SURFACE)
        .border_1()
        .border_color(colors::BORDER)
        .child(
            view()
                .w_full()
                .px(px(10.0))
                .py(px(4.0))
                .rounded(px(radius::CONTROL))
                .bg(move || {
                    if preset.get() == QuickPreset::All {
                        colors::INSET
                    } else {
                        colors::SURFACE
                    }
                })
                .child(
                    text(move || format!("All Instruments ({})", total.get()))
                        .text_size(px(10.5))
                        .text_color(move || colors::TEXT),
                )
                .on_press(move || preset.set(QuickPreset::All)),
        )
        .child(
            view()
                .w_full()
                .px(px(10.0))
                .py(px(4.0))
                .rounded(px(radius::CONTROL))
                .bg(move || {
                    if preset.get() == QuickPreset::Watchlist {
                        colors::INSET
                    } else {
                        colors::SURFACE
                    }
                })
                .child(
                    text(move || format!("Watchlist Only ({})", shortlisted.get().len()))
                        .text_size(px(10.5))
                        .text_color(move || colors::TEXT),
                )
                .on_press(move || preset.set(QuickPreset::Watchlist)),
        )
        .child(
            view()
                .w_full()
                .px(px(10.0))
                .py(px(4.0))
                .rounded(px(radius::CONTROL))
                .bg(colors::SURFACE)
                .child(
                    text("[Clear Watchlist]")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || colors::DOWN),
                )
                .on_press(move || shortlisted.set(Vec::new())),
        )
}

/// One multi-select index row: toggles membership, ✓ when selected.
fn index_option_row(name: &'static str, selection: Signal<Vec<String>>) -> View {
    view()
        .px(px(10.0))
        .py(px(3.0))
        .rounded(px(radius::CONTROL))
        .bg(move || {
            if selection.get().iter().any(|entry| entry.as_str() == name) {
                colors::ACCENT
            } else {
                colors::SURFACE
            }
        })
        .child(
            text(move || {
                if selection.get().iter().any(|entry| entry.as_str() == name) {
                    format!("✓ {name}")
                } else {
                    name.to_string()
                }
            })
            .text_size(px(10.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(move || {
                if selection.get().iter().any(|entry| entry.as_str() == name) {
                    colors::ON_ACCENT
                } else {
                    colors::TEXT
                }
            }),
        )
        .on_press(move || {
            selection.update(|current| {
                if let Some(position) = current.iter().position(|entry| entry.as_str() == name) {
                    current.remove(position);
                } else {
                    current.push(name.to_string());
                }
            });
        })
}

/// Index drawer: two labelled groups of multi-select index toggles.
fn index_drawer(selection: Signal<Vec<String>>) -> View {
    view()
        .flex_col()
        .gap(px(6.0))
        .w_full()
        .px(px(8.0))
        .py(px(6.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::SURFACE)
        .border_1()
        .border_color(colors::BORDER)
        .child(
            text("Broad Market")
                .text_size(px(10.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || colors::MUTED),
        )
        .child(
            view().flex_row().gap(px(6.0)).children(
                BROAD_MARKET_INDEXES
                    .iter()
                    .map(|name| index_option_row(name, selection)),
            ),
        )
        .child(
            text("Sectoral")
                .text_size(px(10.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || colors::MUTED),
        )
        .child(
            view().flex_row().gap(px(6.0)).children(
                SECTORAL_INDEXES
                    .iter()
                    .map(|name| index_option_row(name, selection)),
            ),
        )
}

/// Queue a non-empty AI prompt: log it for the honba-nou AST synthesizer,
/// bump the local counter, clear the draft, and collapse the prompt row.
fn queue_ai_prompt(query: &str, draft: Signal<String>, sent: Signal<usize>, open: Signal<bool>) {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return;
    }
    tracing::info!(
        query = trimmed,
        "screener AI prompt queued for honba-nou AST synthesizer"
    );
    sent.update(|count| *count += 1);
    draft.set(String::new());
    open.set(false);
}

/// `✦ AI` prompt row: full-width input + send chip under the filter bar.
fn ai_prompt_row(open: Signal<bool>, draft: Signal<String>, sent: Signal<usize>) -> View {
    let ai_input = HfInput::element(HfInputProps {
        value: Prop::bind(move || draft.get()),
        placeholder: Prop::from(Some(AI_PLACEHOLDER.to_string())),
        disabled: Prop::from(false),
    })
    .on_event(move |event: &HfInputEvent| {
        let HfInputEvent::Change(value) = event;
        draft.set(value.clone());
    });

    let send_chip = HfButton::element(ghost_button_props())
        .child(text("Send"))
        .on_event(move |event: &HfButtonEvent| {
            if *event == HfButtonEvent::Press {
                let query = draft.get();
                queue_ai_prompt(&query, draft, sent, open);
            }
        });

    view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .w_full()
        .px(px(8.0))
        .py(px(6.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::SURFACE)
        .border_1()
        .border_color(colors::BORDER)
        .child(view().flex_1().child(ai_input))
        .child(send_chip)
}
