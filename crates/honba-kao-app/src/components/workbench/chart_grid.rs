//! The workspace grid, ported from the labs `multi-chart-grid.tsx`: a CSS
//! grid whose template class follows the layout preset, hosting the panes.
//! (Grid shapes live in `style/workbench.css` as `wb-grid--*` named areas.)

use leptos::prelude::*;

use super::pane_chart::ChartPane;
use crate::stores::workbench_store::WorkbenchStore;

#[component]
pub fn MultiChartGrid() -> impl IntoView {
    let store = WorkbenchStore::take();

    view! {
        <div class=move || format!("wb-grid wb-grid--{}", store.layout.layout.get().as_id())>
            <ChartPane index=0 store=store />
            <ChartPane index=1 store=store />
            <ChartPane index=2 store=store />
            <ChartPane index=3 store=store />
        </div>
    }
}
