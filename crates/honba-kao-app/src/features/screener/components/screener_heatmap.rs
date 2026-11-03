//! `ScreenerHeatmap` — sector-grouped treemap-style market heatmap with
//! cap-weighted tile sizing and `change_pct`-interpolated heat colors.

use crate::features::screener::filters::{format_change_pct, format_mcap_rs_cr};
use crate::features::screener::heatmap::{SectorHeatGroup, cap_share_pct, group_sectors, heat_hex};
use crate::features::screener::types::ScreenerEquityRecord;
use leptos::prelude::*;

#[component]
pub fn ScreenerHeatmap(
    records: Signal<Vec<ScreenerEquityRecord>>,
    active_symbol: RwSignal<String>,
    drawer_open: RwSignal<bool>,
) -> impl IntoView {
    let groups: Memo<Vec<SectorHeatGroup>> = Memo::new(move |_| group_sectors(&records.get()));

    let sector_cards = move || {
        groups
            .get()
            .into_iter()
            .map(|group| {
                let cap_label = format_mcap_rs_cr(group.total_cap_cr);
                let avg_label = format_change_pct(group.avg_change_pct);
                let header = format!("{cap_label} · {avg_label}");
                let sector_name = group.sector.clone();
                let total_cap = group.total_cap_cr;
                let tiles = group
                    .members
                    .into_iter()
                    .map(|member| {
                        let share = cap_share_pct(member.market_cap_cr, total_cap).to_string();
                        let tile_style = format!(
                            "background-color: {}; flex-grow: {}; flex-basis: 0; overflow: hidden;",
                            heat_hex(member.change_pct),
                            share
                        );
                        let sym_display = member.symbol.clone();
                        let sym_state = member.symbol.clone();
                        let chg_label = format_change_pct(member.change_pct);
                        let cap_cat = member.market_cap_category.clone();
                        let tile_class = move || {
                            if active_symbol.get() == sym_state {
                                "min-w-[72px] shrink cursor-pointer rounded border border-black/20 p-2 text-white ring-2 ring-white/80"
                            } else {
                                "min-w-[72px] shrink cursor-pointer rounded border border-black/20 p-2 text-white opacity-95 hover:opacity-100"
                            }
                        };
                        view! {
                            <div
                                class=tile_class
                                style=tile_style
                                on:click=move |_| {
                                    active_symbol.set(sym_display.clone());
                                    drawer_open.set(true);
                                }
                            >
                                <div class="truncate font-mono text-xs font-semibold">
                                    {member.symbol.clone()}
                                </div>
                                <div class="truncate font-mono text-[10px] opacity-90">{chg_label}</div>
                                <div class="truncate text-[10px] opacity-75">{cap_cat}</div>
                            </div>
                        }
                        .into_any()
                    })
                    .collect::<Vec<AnyView>>();
                view! {
                    <div class="hf-sector-card flex-1 min-w-[280px] max-w-[460px] rounded-lg border border-border-soft bg-surface-base p-2">
                        <div class="mb-2 flex items-baseline justify-between gap-2">
                            <span class="truncate text-[11px] font-semibold uppercase tracking-wide text-text-muted">
                                {sector_name}
                            </span>
                            <span class="shrink-0 font-mono text-[11px] tabular-nums text-text-muted">
                                {header}
                            </span>
                        </div>
                        <div class="flex gap-1">{tiles}</div>
                    </div>
                }
                .into_any()
            })
            .collect::<Vec<AnyView>>()
    };

    // Compact legend: 7 documented heat palette steps, right-aligned.
    let legend = view! {
        <div class="mt-2 flex flex-wrap items-center justify-end gap-x-3 gap-y-1 text-[10px] text-text-muted">
            <span class="flex items-center gap-1">
                <span class="h-[3px] w-7 rounded-sm" style="background-color: #e11d48"></span>
                "< -3%"
            </span>
            <span class="flex items-center gap-1">
                <span class="h-[3px] w-7 rounded-sm" style="background-color: #f43f5e"></span>
                "-3%..-1%"
            </span>
            <span class="flex items-center gap-1">
                <span class="h-[3px] w-7 rounded-sm" style="background-color: #450a0a"></span>
                "-1%..0%"
            </span>
            <span class="flex items-center gap-1">
                <span class="h-[3px] w-7 rounded-sm" style="background-color: #1e293b"></span>
                "0%"
            </span>
            <span class="flex items-center gap-1">
                <span class="h-[3px] w-7 rounded-sm" style="background-color: #064e3b"></span>
                "0%..+1%"
            </span>
            <span class="flex items-center gap-1">
                <span class="h-[3px] w-7 rounded-sm" style="background-color: #10b981"></span>
                "+1%..+3%"
            </span>
            <span class="flex items-center gap-1">
                <span class="h-[3px] w-7 rounded-sm" style="background-color: #059669"></span>
                "> +3%"
            </span>
        </div>
    };

    view! {
        <div class="hf-screener-heatmap flex h-full w-full flex-col p-3">
            {move || {
                if groups.get().is_empty() {
                    view! {
                        <div class="flex flex-1 items-center justify-center text-xs text-text-muted">
                            "No instruments match the active filters"
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="flex flex-wrap gap-3">{sector_cards}</div>
                    }
                    .into_any()
                }
            }}
            {legend}
        </div>
    }
}
