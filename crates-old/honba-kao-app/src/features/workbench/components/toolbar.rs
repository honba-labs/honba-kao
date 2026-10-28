//! Top toolbar, ported from the labs `workbench-toolbar.tsx`: layout picker
//! (workspace grid), engine kill switch, order-ticket and drawer toggles plus
//! the status/error pill.

use leptos::prelude::*;

use super::layouts::{LAYOUT_PRESETS, LayoutIcon, preset_for};
use crate::features::workbench::stores::book_store::TradingState;
use crate::features::workbench::stores::workbench_store::WorkbenchStore;
use honba_kao_ui_core::{HfButton, dismiss_on_outside_click};

#[component]
pub fn WorkbenchToolbar() -> impl IntoView {
    let store = WorkbenchStore::take();

    let grid_menu_open = RwSignal::new(false);
    let grid_menu_host = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside_click(grid_menu_open, grid_menu_host);

    let state_menu_open = RwSignal::new(false);
    let state_menu_host = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside_click(state_menu_open, state_menu_host);

    view! {
        <div class="wb-toolbar">
            // -- Left: workspace grid picker --------------------------------
            <div class="wb-toolbar__left">
                <div class="wb-toolbar__grid-label">
                    <span class="wb-toolbar__eyebrow">"Workspace Grid"</span>
                    {move || {
                        let preset = store.layout.layout.get();
                        view! { <span class="wb-toolbar__preset">{preset_for(preset).label}</span> }
                    }}
                </div>
                <div class="wb-toolbar__menu" node_ref=grid_menu_host>
                    <button
                        class="wb-btn wb-btn--chrome"
                        title=move || preset_for(store.layout.layout.get()).label
                        aria-haspopup="menu"
                        on:click=move |_| grid_menu_open.update(|open| *open = !*open)
                    >
                        <span class="wb-btn__icon">
                            {move || view! { <LayoutIcon layout=store.layout.layout.get() /> }}
                        </span>
                        <span class="wb-caret">"▾"</span>
                    </button>
                    <Show when=move || grid_menu_open.get()>
                        <div class="wb-menu" role="menu">
                            {LAYOUT_PRESETS
                                .iter()
                                .map(|preset| {
                                    let layout = preset.layout;
                                    let selected = move || store.layout.layout.get() == layout;
                                    view! {
                                        <button
                                            class=move || {
                                                if selected() {
                                                    "wb-menu__item wb-menu__item--selected"
                                                } else {
                                                    "wb-menu__item"
                                                }
                                            }
                                            role="menuitem"
                                            title=preset.label
                                            on:click=move |_| {
                                                store.layout.set_layout(layout);
                                                grid_menu_open.set(false);
                                            }
                                        >
                                            <span class="wb-btn__icon">
                                                {view! { <LayoutIcon layout=layout /> }}
                                            </span>
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </Show>
                </div>
            </div>

            // -- Right: status, engine, toggles ------------------------------
            <div class="wb-toolbar__right">
                {move || match store.book.status.get() {
                    Some(status) => {
                        let text = status.text().to_string();
                        view! {
                            <HfButton
                                unstyled=true
                                class=status.as_class()
                                title="Dismiss"
                                on_click=Callback::new(move |_| store.book.dismiss_status())
                            >
                                {text}
                            </HfButton>
                        }
                            .into_any()
                    }
                    None => ().into_any(),
                }}

                // Engine-state menu: the badge opens the three operator states
                // (`PUT /trading-state` accepts any transition), so Reducing is
                // reachable without leaving the toolbar.
                <div class="wb-toolbar__menu" node_ref=state_menu_host>
                    <button
                        class=move || format!("wb-engine wb-engine--{}", store.book.trading_state.get().as_id())
                        title="Trading state (click to change)"
                        aria-haspopup="menu"
                        on:click=move |_| state_menu_open.update(|open| *open = !*open)
                    >
                        {move || format!("Engine: {}", store.book.trading_state.get().as_id())}
                        <span class="wb-caret">"▾"</span>
                    </button>
                    <Show when=move || state_menu_open.get()>
                        <div class="wb-menu wb-menu--right" role="menu">
                            {[
                                (TradingState::Active, "Active — full execution"),
                                (TradingState::Reducing, "Reducing — exits only"),
                                (TradingState::Halted, "Halted — no orders"),
                            ]
                                .into_iter()
                                .map(|(state, label)| {
                                    let selected = move || store.book.trading_state.get() == state;
                                    view! {
                                        <button
                                            class=move || {
                                                if selected() {
                                                    "wb-menu__item wb-menu__item--selected"
                                                } else {
                                                    "wb-menu__item"
                                                }
                                            }
                                            role="menuitem"
                                            title=label
                                            on:click=move |_| {
                                                store.book.set_trading_state(state, None);
                                                state_menu_open.set(false);
                                            }
                                        >
                                            <span class="wb-engine wb-engine--state-menu-label">
                                                {state.as_id()}
                                            </span>
                                            <span class="wb-menu__hint">{label}</span>
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </Show>
                </div>

                {move || {
                    let halted = store.book.trading_state.get() == TradingState::Halted;
                    let button_class = if halted {
                        "wb-btn wb-btn--solid wb-btn--buy"
                    } else {
                        "wb-btn wb-btn--solid wb-btn--sell"
                    };
                    let label = if halted { "Resume Trading" } else { "Kill Switch" };
                    view! {
                        <button
                            class=button_class
                            on:click=move |_| store.book.toggle_kill_switch()
                        >
                            {label}
                        </button>
                    }
                }}

                <button
                    class="wb-btn wb-btn--chrome"
                    title="Search symbol"
                    on:click=move |_| store.layout.open_symbol_search(None)
                >
                    "Search"
                </button>

                <button
                    class=move || {
                        if store.draft.ticket_open.get() {
                            "wb-btn wb-btn--chrome wb-btn--engaged"
                        } else {
                            "wb-btn wb-btn--chrome"
                        }
                    }
                    title=move || {
                        if store.draft.ticket_open.get() {
                            "Collapse Order Ticket"
                        } else {
                            "Expand Order Ticket"
                        }
                    }
                    on:click=move |_| store.draft.toggle_order_ticket()
                >
                    "Ticket"
                </button>

                <button
                    class="wb-btn wb-btn--chrome"
                    title=move || {
                        if store.layout.drawer_open.get() {
                            "Collapse Bottom Drawer"
                        } else {
                            "Expand Bottom Drawer"
                        }
                    }
                    on:click=move |_| store.layout.toggle_drawer()
                >
                    <span class="wb-caret">{move || if store.layout.drawer_open.get() { "▾" } else { "▴" }}</span>
                </button>
            </div>
        </div>
    }
}
