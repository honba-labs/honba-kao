//! Screener — the AST-filtered scan board (screener-design-doc.md v2.0).
//!
//! The page owns the screening state (asset class, quick preset, view
//! tab, heatmap/or-table view mode, search, granular criteria, sort,
//! shortlist) as signals; one effect recomputes rows over the fixture
//! universes — `ScreenerRecord` for the grid and the stock-only board
//! for the heatmap — and a sibling effect resets the visible columns
//! when the class or the tab changes. The filter bar sits above a
//! resizable split: the high-density grid or the market heatmap on the
//! left, the detail drawer on the right, the filter and column modals
//! overlaying the page while open.
//!
//! Reactivity discipline (doc §3): the drawer and the two modals mount
//! through `when` regions bound to the *signals themselves* — never
//! `.get()` — and every panel lives in a `dynamic` region so its
//! synchronous reads re-run the build after each press.

use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_base::{h_resizable, resizable_panel};
use nsh_core::prelude::*;
use std::rc::Rc;

use super::column_modal::ScreenerColumnModal;
use super::detail_drawer::ScreenerDetailDrawer;
use super::filter_bar::{MarketRegion, ScreenerFilterBar};
use super::filter_modal::ScreenerFilterModal;
use super::open_screen_modal::ScreenerOpenScreenModal;
use super::screener_heatmap::{ScreenerHeatmap, ScreenerViewMode};
use super::table_grid::{
    ScreenerTableGrid, default_visible_columns_for, filter_instruments, filter_records,
    screener_records, screener_universe, sort_records,
};
use super::types::{
    FilterCriteria, QuickPreset, ScreenerAssetType, ScreenerInstrument, ScreenerRecord,
    ScreenerViewTab, SortState,
};

#[derive(Debug, Clone, PartialEq, Props)]
pub struct ScreenerPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Screener — filter the fixture universes into a live board.
#[component]
pub fn ScreenerPage(_props: ScreenerPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    // Two fixture universes: the record union drives the grid + drawer,
    // the stock list the heatmap's cap-weighted tiles.
    let universe: Vec<ScreenerInstrument> = screener_universe();
    let records_universe: Vec<ScreenerRecord> = screener_records();

    let asset: Signal<ScreenerAssetType> = signal(ScreenerAssetType::Stocks);
    let preset: Signal<QuickPreset> = signal(QuickPreset::All);
    let tab: Signal<ScreenerViewTab> = signal(ScreenerViewTab::Overview);
    let view_mode: Signal<ScreenerViewMode> = signal(ScreenerViewMode::Table);
    let group_by_sector: Signal<bool> = signal(true);
    let search: Signal<String> = signal(String::new());
    let criteria: Signal<FilterCriteria> = signal(FilterCriteria::default());
    let sort_state: Signal<Option<SortState>> = signal(None);
    let active_symbol: Signal<String> = signal(universe[0].symbol.clone());
    let drawer_open: Signal<bool> = signal(false);
    let shortlisted: Signal<Vec<String>> = signal(Vec::new());
    let filter_modal_open: Signal<bool> = signal(false);
    let column_modal_open: Signal<bool> = signal(false);
    let screen_modal_open: Signal<bool> = signal(false);
    let screen_save_open: Signal<bool> = signal(false);
    let market_region: Signal<MarketRegion> = signal(MarketRegion::India);
    let index_selection: Signal<Vec<String>> = signal(Vec::new());
    let visible_columns: Signal<Vec<String>> = signal(default_visible_columns_for(
        ScreenerAssetType::Stocks,
        ScreenerViewTab::Overview,
    ));
    let matches: Signal<usize> = signal(0usize);
    let total: Signal<usize> = signal(universe.len());
    let rows: Signal<Vec<ScreenerRecord>> = signal(Vec::new());
    let stock_rows: Signal<Vec<ScreenerInstrument>> = signal(Vec::new());

    // One roster for the page: the filter-bar chips and the filter modal
    // share the same "All" + sorted unique sectors list.
    let mut sector_roster: Vec<String> = vec!["All".to_string()];
    let mut unique = universe
        .iter()
        .map(|instrument| instrument.sector.clone())
        .collect::<Vec<_>>();
    unique.sort();
    unique.dedup();
    sector_roster.extend(unique);
    let sectors: Signal<Vec<String>> = signal(sector_roster);

    // Rows + match count: immediate on mount, then exactly when asset,
    // preset, search, criteria, the shortlist, or the active sort
    // change — the rows arrive pre-filtered AND pre-sorted (the corriger
    // for the stock grid's display never applied the sort state).
    let records_for_rows: Vec<ScreenerRecord> = records_universe.clone();
    let universe_for_heat: Vec<ScreenerInstrument> = universe.clone();
    effect(move || {
        let asset_now = asset.get();
        let preset_now = preset.get();
        let search_now = search.get();
        let criteria_now = criteria.get();
        let shortlist_now = shortlisted.get();
        let sort_now = sort_state.get();

        let filtered = filter_records(
            &records_for_rows,
            asset_now,
            preset_now,
            search_now.as_str(),
            &criteria_now,
            shortlist_now.as_slice(),
        );
        let count = filtered.len();
        let sorted = sort_records(filtered, sort_now.as_ref());

        let heat = filter_instruments(
            &universe_for_heat,
            preset_now,
            search_now.as_str(),
            &criteria_now,
            shortlist_now.as_slice(),
        );
        let class_total = records_for_rows
            .iter()
            .filter(|record| record.class() == asset_now)
            .count();
        batch(move || {
            rows.set(sorted);
            matches.set(count);
            total.set(class_total);
            stock_rows.set(heat);
        });
    });

    // A tab (or asset-class) switch resets the visible columns to that
    // view's defaults.
    effect(move || {
        let cols = default_visible_columns_for(asset.get(), tab.get());
        visible_columns.set(cols);
    });

    let on_open: Rc<dyn Fn(&'static str)> = Rc::new(|path| {
        crate::router::chrome::app_shell::open_path(path);
    });

    // Screen-catalog hotkeys (design doc §5.2): `.` opens the Open
    // Screen dialog; Shift+N opens it straight on the save sheet.
    on_key(".", move |_| {
        screen_save_open.set(false);
        screen_modal_open.update(|open| *open = !*open);
    });
    on_key("shift+n", move |_| {
        screen_save_open.set(true);
        screen_modal_open.set(true);
    });

    // Left workspace: the whole panel swaps between the table grid and
    // the market heatmap at the view mode; every panel body then
    // re-runs on its own signal reads (rows, tab, sort, shortlist…).
    HfPage::element(HfPageProps {
        title: Prop::from(Some("Screener".to_string())),
        subtitle: Prop::from(Some(
            "Institutional screener: filter the fixture universes by asset class, preset, and granular criteria".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(ScreenerFilterBar(
        asset,
        preset,
        tab,
        search,
        criteria,
        matches,
        total,
        sectors,
        filter_modal_open,
        column_modal_open,
        screen_modal_open,
        rows,
        view_mode,
        group_by_sector,
        market_region,
        index_selection,
        shortlisted,
    ))
    .child(
        h_resizable()
            .w_full()
            .h_full()
            .child(resizable_panel().child(dynamic(move || {
                match view_mode.get() {
                    ScreenerViewMode::Table => ScreenerTableGrid(
                        rows,
                        asset,
                        tab,
                        sort_state,
                        active_symbol,
                        drawer_open,
                        shortlisted,
                        visible_columns,
                        criteria,
                    ),
                    ScreenerViewMode::Heatmap => {
                        ScreenerHeatmap(stock_rows, active_symbol, drawer_open, group_by_sector)
                    }
                }
            })))
            .child(
                resizable_panel()
                    .size(px(360.0))
                    .size_range(px(280.0)..px(480.0))
                    .visible(move || drawer_open.get())
                    .child(when(drawer_open, move || {
                        let records_for_drawer = records_universe.clone();
                        let on_open_for_drawer = on_open.clone();
                        dynamic(move || {
                            ScreenerDetailDrawer(
                                records_for_drawer.clone(),
                                active_symbol,
                                shortlisted,
                                drawer_open,
                                on_open_for_drawer.clone(),
                            )
                        })
                    })),
            ),
    )
    // The modals overlay the page; their `when` regions bind the open
    // signals themselves (doc §3 — a static `.get()` never mounts).
    .child(when(filter_modal_open, move || {
        dynamic(move || ScreenerFilterModal(criteria, sectors, filter_modal_open))
    }))
    .child(when(column_modal_open, move || {
        dynamic(move || ScreenerColumnModal(tab, visible_columns, column_modal_open))
    }))
    .child(when(screen_modal_open, move || {
        dynamic(move || {
            ScreenerOpenScreenModal(
                screen_modal_open,
                screen_save_open,
                asset,
                tab,
                preset,
                criteria,
                visible_columns,
            )
        })
    }))
}
