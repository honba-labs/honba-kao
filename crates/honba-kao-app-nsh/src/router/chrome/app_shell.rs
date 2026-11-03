//! The app's shell (the route clusters): the workstation chrome from
//! 700 pt wide and the pool's phone layout below that.
//!
//! Every page of `crate::router::routes::route_element` is a shell
//! entry. From 700 pt the desktop is the workstation chrome of
//! `docs/workstation-chrome-instruction.md`: the 54 pt icon-only
//! [`nav_rail`] (one button per cluster), the 44 pt [`header_bar`]
//! (breadcrumb, the cluster's sub-navigation pills, the Ctrl+G / Ctrl+J
//! triggers, the connection LEDs) and the full-bleed pane of the open
//! entry — the pool's 264 pt demo sidebar and its 64 pt catalog banner
//! are retired (`hf_shell_desktop` unused). Below that, the pool's
//! [`hf_shell_phone`] shows this file's [`index`].
//!
//! The entry open last is kept in nsh storage under `lastRoute` and
//! opened again at launch ([`hf_shell_restore`] for the pool's copy;
//! [`seed_selection`] seeds the workstation chrome's mirror
//! [`WorkstationSelected`] — the pool's counterpart is private). Every
//! opening goes through [`hf_shell_open`], which selects in the pool
//! shell too, so its storage keeper (it only writes the phone's pushed
//! entries) stays consistent. The global GoBar routes through
//! [`open_path`].

use std::rc::Rc;

use nsh_base::nav_page;
use nsh_core::{
    element::{AnyElement, Screen, SharedString, Signal},
    prelude::*,
    storage::Storage,
};

use honba_kao_ui_core::icons::canvas_icons::{IconName, IconTile, icon};
use honba_kao_ui_core::nav::{Route, depth, pop_to_top, push_route, reset, routes};
use honba_kao_ui_core::primitive::prompt::{HfStatusLed, HfStatusLedProps, StatusTone};
use honba_kao_ui_core::templates::shell::{
    HfResponsiveShellSpec, HfShellEntry, HfShellGroup, hf_shell_open, hf_shell_phone,
    hf_shell_restore,
};
use honba_kao_ui_core::theme::tokens::{MONO, colors, radius};

use crate::router::chrome::{header_bar, nav_rail};
use crate::router::paths::*;
use crate::router::routes::route_element;

/// The chrome's title face: the family `assets/style/fonts.css` loads
/// for the app (`Suisse Intl`).
pub(crate) const SUISSE: &str = "Suisse Intl";

// ----------------------------------------------------------------------------
// The footnote's status: fn pointers cannot `use_context`, so the shell's
// footnote renders from an nsh global the chrome installs from the injected
// client context.
// ----------------------------------------------------------------------------

/// Connection status for the shell's footnote (the fn-pointer slot the
/// pool's spec leaves for context). Installed by `AppChrome` from
/// [`crate::shared::contexts::HonbaClientContext`]'s signals.
///
/// Clonable/Default because nsh globals are `Signal<T: Clone + Default>`.
#[derive(Clone, Default)]
pub(crate) struct ChromeStatus {
    /// Whether the OMS bridge is connected.
    pub(crate) oms: bool,
    /// Whether the market-data feed is live.
    pub(crate) feed: bool,
    /// The last round-trip latency sample, if any.
    pub(crate) latency: Option<u64>,
}

/// Publishes the current connection status for [`footnote`] to render.
pub(crate) fn install_status(oms: bool, feed: bool, latency: Option<u64>) {
    global::<ChromeStatus>().set(ChromeStatus { oms, feed, latency });
}

// ----------------------------------------------------------------------------
// The workstation chrome (docs/workstation-chrome-instruction.md): the
// desktop layout replaces the pool's sidebar + pane with [`nav_rail`]
// + [`header_bar`].
// ----------------------------------------------------------------------------

/// The workstation chrome's open entry — the [`nav_rail`]'s active
/// cluster, the [`header_bar`]'s breadcrumb/pills and the content
/// pane's source. A mirror of the pool shell's selection: the pool's
/// counterpart (`templates::shell`'s) is private, so every opening goes
/// through [`select_entry`] — it sets this mirror and calls
/// [`hf_shell_open`], which selects in the pool shell too and keeps the
/// pool's storage keeper consistent.
#[derive(Clone, Copy)]
pub(crate) struct WorkstationSelected(pub(crate) &'static HfShellEntry);

static EMPTY_ENTRY: [HfShellEntry; 1] = [HfShellEntry {
    name: "",
    detail: "",
    icon: IconName::System,
    content: || view().into_element(),
}];

impl Default for WorkstationSelected {
    fn default() -> Self {
        Self(&EMPTY_ENTRY[0])
    }
}

impl PartialEq for WorkstationSelected {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

/// Whether the workstation chrome is showing (the phone layout carries
/// its pick over the index when it returns — the pool's own
/// `hf_shell_desktop` does this for its sidebar).
#[derive(Default, Clone, Copy, PartialEq)]
struct WorkstationWasDesktop(bool);

/// The overlays' openers (the [`header_bar`]'s buttons toggle them; the
/// builder slots cannot `use_context`, so `AppChrome` installs them).
#[derive(Clone, Default)]
pub(crate) struct WorkspaceHotkeys {
    /// The GoBar's opener.
    pub(crate) gobar: Option<Signal<bool>>,
    /// The agent console's opener.
    pub(crate) agent: Option<Signal<bool>>,
}

/// The workstation chrome's open entry (the mirror [`select_entry`]
/// keeps; the builders read it).
pub(crate) fn workstation_entry() -> Signal<WorkstationSelected> {
    global::<WorkstationSelected>()
}

/// The overlays' openers' global (the [`header_bar`]'s buttons read
/// it).
pub(crate) fn workspace_hotkeys() -> Signal<WorkspaceHotkeys> {
    global::<WorkspaceHotkeys>()
}

/// The shell's groups (the [`nav_rail`]'s clusters).
pub(crate) fn workstation_groups() -> &'static [HfShellGroup] {
    GROUPS
}

/// Selects `entry` for the workstation chrome and opens it in the pool
/// shell too (one storage write, both shells consistent).
pub(crate) fn select_entry(entry: &'static HfShellEntry) {
    global::<WorkstationSelected>().set(WorkstationSelected(entry));
    hf_shell_open(&APP, entry);
}

// ----------------------------------------------------------------------------
// The spec
// ----------------------------------------------------------------------------

/// The app's shell spec: the route clusters as groups (every page of
/// `crate::router::routes::route_element` a shell entry), `lastRoute` in
/// the app's storage, the phone index and the brand's title face.
static APP: HfResponsiveShellSpec = HfResponsiveShellSpec {
    storage_key: "lastRoute",
    groups: GROUPS,
    root: index,
    route_of: app_route,
    brand,
    footnote,
    display_font: || SharedString::from(SUISSE),
};

/// Restores the page open when the app last ran; then keeps the one
/// open in the app's storage (the pool's shell handles it; the
/// workstation chrome's mirror is seeded beside it).
pub(crate) fn restore() {
    hf_shell_restore(&APP);
}

/// Seeds the workstation chrome's mirror with the entry storage keeps
/// under `lastRoute` (the same pick [`hf_shell_restore`] made).
pub(crate) fn seed_selection() {
    let stored = Storage::get_item(APP.storage_key).and_then(|name| {
        GROUPS
            .iter()
            .flat_map(|g| g.entries)
            .find(|entry| entry.name == name)
    });
    let entry = stored.unwrap_or(&GROUPS[0].entries[0]);
    global::<WorkstationSelected>().set(WorkstationSelected(entry));
}

/// Installs the overlays' openers (`AppChrome` does, once per build).
pub(crate) fn install_hotkeys(gobar: Signal<bool>, agent: Signal<bool>) {
    global::<WorkspaceHotkeys>().set(WorkspaceHotkeys {
        gobar: Some(gobar),
        agent: Some(agent),
    });
}

/// The phone layout: the index, pages pushed over it; the entry the
/// workstation showed returns open.
pub(crate) fn phone() -> AnyElement {
    if global::<WorkstationWasDesktop>().get_untracked().0 {
        let entry = global::<WorkstationSelected>().get_untracked().0;
        if !entry.name.is_empty() {
            reset(vec![(APP.route_of)(entry)]);
        }
    }
    hf_shell_phone(&APP)
}

/// The desktop layout: the workstation chrome — the 54 pt icon rail,
/// the 44 pt header and the full-bleed pane of the open entry (no 64 pt
/// catalog banner; the 264 pt sidebar is retired).
pub(crate) fn desktop_layout() -> AnyElement {
    // The entry the phone layout pushed: carried into the workstation
    // chrome (the pool's `hf_shell_desktop` does this for its sidebar).
    if let Some(entry) = untracked(selected_pushed_entry) {
        select_entry(entry);
    }
    if depth() > 0 {
        pop_to_top();
    }
    global::<WorkstationWasDesktop>().set(WorkstationWasDesktop(true));
    view()
        .flex_1()
        .flex_row()
        .bg(colors::BACKGROUND)
        .child(nav_rail::nav_rail())
        .child(
            view()
                .flex_1()
                .flex_col()
                .overflow_hidden()
                .child(header_bar::header_bar())
                .child(view().flex_1().child(dynamic(move || {
                    let content = workstation_entry().get().0.content;
                    untracked(content)
                }))),
        )
        .into_element()
}

/// The entry the phone layout has pushed, if any (mirrors the pool
/// shell's `pushed_entry`).
fn selected_pushed_entry() -> Option<&'static HfShellEntry> {
    routes().with(|stack| {
        GROUPS
            .iter()
            .flat_map(|g| g.entries)
            .find(|entry| stack.0.iter().any(|r| *r.key == *entry.key()))
    })
}

/// The entry for a `crate::router::paths::ROUTE_*` path (a hand-kept
/// table: the GoBar's destinations), if the shell has one.
pub(crate) fn entry_for_path(path: &str) -> Option<&'static HfShellEntry> {
    PATH_ENTRIES
        .iter()
        .find(|(entry_path, _)| *entry_path == path)
        .map(|(_, entry)| entry())
}

/// Opens the page a GoBar pick or typed query names: selected in the
/// workstation chrome, pushed over the index on phones.
pub(crate) fn open_path(path: &str) {
    if let Some(entry) = entry_for_path(path) {
        select_entry(entry);
    }
}

// ----------------------------------------------------------------------------
// The routes as shell entries
// ----------------------------------------------------------------------------

/// A page's screen as the phone layout pushes it over the index: its
/// name in the header over its content (the pool's entry type has no
/// header of its own).
fn app_route(entry: &'static HfShellEntry) -> Route {
    let (name, content) = (entry.name, entry.content);
    Route {
        key: entry.key().into(),
        screen: Rc::new(move || {
            nav_page()
                .title(name)
                .child(view().flex_1().bg(colors::BACKGROUND).child(content()))
        }),
    }
}

/// The app's pages, grouped by route cluster (mirrors
/// [`route_element`]'s own sections). Each entry's content is the routed
/// element of its `ROUTE_*` path.
static GROUPS: &[HfShellGroup] = &[
    HfShellGroup {
        title: "System",
        entries: &[
            HfShellEntry {
                name: "Home",
                detail: "Command hub: session status, quick actions, workspace overview",
                icon: IconName::System,
                content: || route_element(ROUTE_HOME),
            },
            HfShellEntry {
                name: "Landing",
                detail: "Institutional overview surfaces for new sessions and demos",
                icon: IconName::Templates,
                content: || route_element(ROUTE_LANDING),
            },
            HfShellEntry {
                name: "Auth",
                detail: "Broker sign-in: API keys and headless TOTP sessions",
                icon: IconName::Check,
                content: || route_element(ROUTE_AUTH),
            },
            HfShellEntry {
                name: "Account",
                detail: "Profile, preferences, risk defaults and workspace settings",
                icon: IconName::Storage,
                content: || route_element(ROUTE_ACCOUNT),
            },
        ],
    },
    HfShellGroup {
        title: "Trading",
        entries: &[
            HfShellEntry {
                name: "Cockpit",
                detail: "Primary execution desk: charts, ticket, DOM and positions",
                icon: IconName::Stocks,
                content: || route_element(ROUTE_TRADING_COCKPIT),
            },
            HfShellEntry {
                name: "Multi-Chart",
                detail: "Multi-pane chart workstation with linked timeframes",
                icon: IconName::Canvas,
                content: || route_element(ROUTE_TRADING_MULTI_CHART),
            },
            HfShellEntry {
                name: "DOM",
                detail: "Depth-of-market ladder with click-to-trade routing",
                icon: IconName::Lists,
                content: || route_element(ROUTE_TRADING_DOM),
            },
            HfShellEntry {
                name: "Tape",
                detail: "Time & sales tape with aggressor flags and filters",
                icon: IconName::Scroll,
                content: || route_element(ROUTE_TRADING_TAPE),
            },
            HfShellEntry {
                name: "MTA",
                detail: "Multi-timeframe alignment grid across watched scrips",
                icon: IconName::Data,
                content: || route_element(ROUTE_TRADING_MTA),
            },
            HfShellEntry {
                name: "Compare",
                detail: "Side-by-side relative performance and spread study",
                icon: IconName::Components,
                content: || route_element(ROUTE_TRADING_COMPARE),
            },
            HfShellEntry {
                name: "Launchpad",
                detail: "Saved layouts and one-press workspace launchers",
                icon: IconName::Plus,
                content: || route_element(ROUTE_TRADING_LAUNCHPAD),
            },
        ],
    },
    HfShellGroup {
        title: "Derivatives",
        entries: &[
            HfShellEntry {
                name: "Option Chain",
                detail: "Live chain with OI, IV and Greeks across strikes",
                icon: IconName::Fno,
                content: || route_element(ROUTE_DERIVATIVES_OPTION_CHAIN),
            },
            HfShellEntry {
                name: "Strategy Builder",
                detail: "Leg builder with payoff preview and margin estimates",
                icon: IconName::Inputs,
                content: || route_element(ROUTE_DERIVATIVES_STRATEGY),
            },
            HfShellEntry {
                name: "Greeks Heatmap",
                detail: "Strike-by-expiry Greeks surface at a glance",
                icon: IconName::Physics,
                content: || route_element(ROUTE_DERIVATIVES_GREEKS),
            },
            HfShellEntry {
                name: "OI Analysis",
                detail: "Open interest distribution, change and max pain",
                icon: IconName::Data,
                content: || route_element(ROUTE_DERIVATIVES_OI),
            },
        ],
    },
    HfShellGroup {
        title: "Portfolio",
        entries: &[
            HfShellEntry {
                name: "Overview",
                detail: "Net positions, P&L, margin and exposure at a glance",
                icon: IconName::Core,
                content: || route_element(ROUTE_PORTFOLIO),
            },
            HfShellEntry {
                name: "Tradebook",
                detail: "Executed trades journal with FIFO tax-lot detail",
                icon: IconName::Scroll,
                content: || route_element(ROUTE_PORTFOLIO_TRADEBOOK),
            },
            HfShellEntry {
                name: "Shadow Account",
                detail: "Paper-tracked mirror account against live fills",
                icon: IconName::Overlays,
                content: || route_element(ROUTE_PORTFOLIO_SHADOW),
            },
            HfShellEntry {
                name: "Lab Optimizer",
                detail: "Allocation optimizer over risk and constraint budgets",
                icon: IconName::Spark,
                content: || route_element(ROUTE_PORTFOLIO_OPTIMIZER),
            },
        ],
    },
    HfShellGroup {
        title: "Wealth",
        entries: &[
            HfShellEntry {
                name: "Mutual Funds",
                detail: "Fund universe with riskometers and overlap analysis",
                icon: IconName::Wealth,
                content: || route_element(ROUTE_WEALTH_MUTUAL_FUNDS),
            },
            HfShellEntry {
                name: "SIP Planner",
                detail: "Step-up SIP schedules with projected corpus curves",
                icon: IconName::Animation,
                content: || route_element(ROUTE_WEALTH_SIP),
            },
            HfShellEntry {
                name: "Asset Allocation",
                detail: "Target weights, drift bands and rebalance proposals",
                icon: IconName::Styles,
                content: || route_element(ROUTE_WEALTH_ALLOCATION),
            },
            HfShellEntry {
                name: "Bonds",
                detail: "Debt ladder with yield-to-maturity and duration view",
                icon: IconName::Star,
                content: || route_element(ROUTE_WEALTH_BONDS),
            },
        ],
    },
    HfShellGroup {
        title: "Quant",
        entries: &[
            HfShellEntry {
                name: "Simulator",
                detail: "Event-driven strategy simulator on historical bars",
                icon: IconName::Play,
                content: || route_element(ROUTE_QUANT_SIMULATOR),
            },
            HfShellEntry {
                name: "Backtesting Deck",
                detail: "Backtest runs with equity curves and metric cards",
                icon: IconName::Pause,
                content: || route_element(ROUTE_QUANT_BACKTESTING),
            },
            HfShellEntry {
                name: "Model Lab",
                detail: "Feature models, fitting runs and walk-forward validation",
                icon: IconName::Spark,
                content: || route_element(ROUTE_QUANT_MODEL_LAB),
            },
            HfShellEntry {
                name: "Stat Lab",
                detail: "Distribution tests, stationarity and regime statistics",
                icon: IconName::Star,
                content: || route_element(ROUTE_QUANT_STAT_LAB),
            },
            HfShellEntry {
                name: "Pair Trading",
                detail: "Cointegration pairs with spread z-score signals",
                icon: IconName::Check,
                content: || route_element(ROUTE_QUANT_PAIR_TRADING),
            },
            HfShellEntry {
                name: "Strategy Studio",
                detail: "Visual node canvas for composing strategy logic",
                icon: IconName::Quant,
                content: || route_element(ROUTE_QUANT_STRATEGY_STUDIO),
            },
        ],
    },
    HfShellGroup {
        title: "Screener",
        entries: &[HfShellEntry {
            name: "Screener",
            detail: "AST-filtered scans across the NSE and F&O universes",
            icon: IconName::Search,
            content: || route_element(ROUTE_SCREENER),
        }],
    },
    HfShellGroup {
        title: "Research",
        entries: &[
            HfShellEntry {
                name: "Security Hub",
                detail: "Per-instrument research console and factor breakdown",
                icon: IconName::Stocks,
                content: || route_element(ROUTE_RESEARCH_SECURITY),
            },
            HfShellEntry {
                name: "Dossiers",
                detail: "Curated company dossiers with filings and notes",
                icon: IconName::Text,
                content: || route_element(ROUTE_RESEARCH_DOSSIERS),
            },
            HfShellEntry {
                name: "Timeline",
                detail: "Corporate action and event chronology per scrip",
                icon: IconName::ArrowDown,
                content: || route_element(ROUTE_RESEARCH_TIMELINE),
            },
            HfShellEntry {
                name: "Markets",
                detail: "Breadth, sector rotation and macro market boards",
                icon: IconName::Data,
                content: || route_element(ROUTE_RESEARCH_MARKETS),
            },
        ],
    },
    HfShellGroup {
        title: "Risk & Ops",
        entries: &[
            HfShellEntry {
                name: "Risk",
                detail: "Pre-trade limits, OTR counters and circuit-breaker state",
                icon: IconName::Bell,
                content: || route_element(ROUTE_RISK),
            },
            HfShellEntry {
                name: "OMS",
                detail: "Order lifecycle state machine and exchange acknowledgements",
                icon: IconName::Form,
                content: || route_element(ROUTE_OMS),
            },
            HfShellEntry {
                name: "Ops",
                detail: "Kill switch, rate limits and operational runbooks",
                icon: IconName::Components,
                content: || route_element(ROUTE_OPS),
            },
            HfShellEntry {
                name: "Position Sizer",
                detail: "Volatility-based sizing across capital and risk budgets",
                icon: IconName::Inputs,
                content: || route_element(ROUTE_POSITION_SIZER),
            },
        ],
    },
];

/// A path's entry accessor: the shell entry its GoBar/navigation state
/// opens.
type EntryOf = fn() -> &'static HfShellEntry;

/// Path → entry accessors for the GoBar's destinations: each returns
/// `&APP.groups[g].entries[i]` by runtime indexing into the statics
/// (the entries the phone index and the sidebar show, never a copy).
const PATH_ENTRIES: &[(&str, EntryOf)] = &[
    (ROUTE_HOME, || &APP.groups[0].entries[0]),
    (ROUTE_TRADING_COCKPIT, || &APP.groups[1].entries[0]),
    (ROUTE_DERIVATIVES_OPTION_CHAIN, || &APP.groups[2].entries[0]),
    (ROUTE_PORTFOLIO, || &APP.groups[3].entries[0]),
    (ROUTE_WEALTH_MUTUAL_FUNDS, || &APP.groups[4].entries[0]),
    (ROUTE_QUANT_SIMULATOR, || &APP.groups[5].entries[0]),
    (ROUTE_SCREENER, || &APP.groups[6].entries[0]),
    (ROUTE_RESEARCH_SECURITY, || &APP.groups[7].entries[0]),
    (ROUTE_RISK, || &APP.groups[8].entries[0]),
];

// ----------------------------------------------------------------------------
// The phone index
// ----------------------------------------------------------------------------

/// An entry's row of the index: its tile, name and detail; a press
/// pushes its screen over the index.
fn index_row(entry: &'static HfShellEntry, first: bool) -> AnyElement {
    view()
        .flex_row()
        .items_center()
        .gap(px(14.0))
        .p_3()
        .cursor_pointer()
        .when(!first, |v| v.border_t_1().border_t_color(colors::LINE))
        .on_press(move || push_route(app_route(entry)))
        .child(IconTile::new(entry.icon).size(38.0))
        .child(
            view()
                .flex_1()
                .gap(px(2.0))
                .child(
                    text(entry.name)
                        .text_size(px(16.0))
                        .font_family(SUISSE)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors::TEXT),
                )
                .child(
                    text(entry.detail)
                        .number_of_lines(2)
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .text_color(colors::MUTED),
                ),
        )
        .child(
            icon()
                .name(IconName::Chevron)
                .size(16.0)
                .tint(colors::FAINT),
        )
        .into_element()
}

/// The index: the route clusters by group; a press pushes one of their
/// screens (the stack's first screen on phones).
fn index() -> Screen {
    let groups: Vec<AnyElement> = GROUPS
        .iter()
        .map(|group| {
            let rows: Vec<AnyElement> = group
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| index_row(entry, i == 0))
                .collect();
            view()
                .gap_2()
                .child(
                    view()
                        .flex_row()
                        .items_baseline()
                        .px_1()
                        .child(
                            text(group.title.to_uppercase())
                                .flex_1()
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::MUTED)
                                .letter_spacing(px(0.4)),
                        )
                        .child(
                            text(group.entries.len().to_string())
                                .text_size(px(12.0))
                                .text_color(colors::FAINT)
                                .font_family(MONO),
                        ),
                )
                .child(
                    view()
                        .bg(colors::SURFACE)
                        .rounded(px(radius::CARD))
                        .border_curve(BorderCurve::Continuous)
                        .border_1()
                        .border_color(colors::BORDER)
                        .overflow_hidden()
                        .children(rows),
                )
                .into_element()
        })
        .collect();
    nav_page().key("index").title("HONBA").child(
        scroll()
            .flex_1()
            .bg(colors::BACKGROUND)
            .child(view().p_4().pb(px(40.0)).gap_5().children(groups)),
    )
}

// ----------------------------------------------------------------------------
// The sidebar's brand and footnote
// ----------------------------------------------------------------------------

/// The sidebar's top: the amber mark and the app's name (the pool's
/// shell gives the brand away).
fn brand() -> AnyElement {
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_5()
        .pt_5()
        .pb_3()
        .child(
            view()
                .size(px(32.0))
                .rounded(px(8.0))
                .bg(colors::ACCENT_FILL)
                .items_center()
                .justify_center()
                .child(
                    text("H")
                        .text_size(px(19.0))
                        .font_family(SUISSE)
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_color(colors::ON_ACCENT),
                ),
        )
        .child(
            view()
                .flex_1()
                .gap(px(1.0))
                .child(
                    text("HONBA")
                        .text_size(px(16.0))
                        .font_family(SUISSE)
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text("Trading Workstation")
                        .text_size(px(12.0))
                        .text_color(colors::MUTED),
                )
                .child(
                    text(format!("Workspace · {}", os().name()))
                        .text_size(px(11.0))
                        .text_color(colors::FAINT),
                ),
        )
        .into_element()
}

/// One status line of the footnote: an LED for `lit`'s condition beside
/// its label (reactive: it re-tints when the global changes).
pub(crate) fn status_row(label: &'static str, lit: fn(&ChromeStatus) -> bool) -> AnyElement {
    let status = global::<ChromeStatus>();
    view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(dynamic(move || {
            HfStatusLed::element(HfStatusLedProps {
                tone: Prop::from(Some(if lit(&status.get()) {
                    StatusTone::Ok
                } else {
                    StatusTone::Offline
                })),
            })
        }))
        .child(
            text(label)
                .text_size(px(12.0))
                .font_family(MONO)
                .text_color(colors::MUTED),
        )
        .into_element()
}

/// The sidebar's bottom: OMS / FEED connection LEDs, the last latency
/// sample and the hotkey hint, from the [`ChromeStatus`] global
/// `AppChrome` installs.
fn footnote() -> AnyElement {
    let status = global::<ChromeStatus>();
    view()
        .px_5()
        .py_3()
        .border_t_1()
        .border_t_color(colors::LINE)
        .gap(px(2.0))
        .child(status_row("OMS", |status| status.oms))
        .child(status_row("FEED", |status| status.feed))
        .child(
            text(move || match status.get().latency {
                Some(ms) => format!("{ms} ms"),
                None => "-- ms".to_string(),
            })
            .text_size(px(12.0))
            .font_family(MONO)
            .text_color(colors::MUTED),
        )
        .child(
            text("Ctrl+G go to · Ctrl+J analyst")
                .text_size(px(11.0))
                .text_color(colors::FAINT),
        )
        .into_element()
}
