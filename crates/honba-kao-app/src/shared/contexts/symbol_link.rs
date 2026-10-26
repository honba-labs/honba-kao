//! Symbol link groups (`A`–`D`): panels in the same group share one symbol.
//!
//! Port of the state model in OpenTerminalUI `contexts/SymbolLinkContext.tsx`.
//! The pure [`LinkBook`] owns the data and is unit tested; [`SymbolLinkContext`]
//! wraps it in a signal so panels re-render when a group symbol or a panel's
//! membership changes. Panels call [`use_link_group`] with a stable panel id.
//!
//! Scope: the context lives in-process (the chrome provides it). OpenTerminalUI
//! also syncs across pop-out windows through a BroadcastChannel; that is not
//! ported yet, and pop-outs are part of Phase 4.5.

use std::collections::BTreeMap;

use honba_kao_ui_core::LinkGroup;
use leptos::prelude::*;

/// Shared state: one symbol per linked group, and each panel's group.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinkBook {
    symbols: [Option<String>; 4],
    panels: BTreeMap<String, LinkGroup>,
}

fn slot(group: LinkGroup) -> Option<usize> {
    LinkGroup::LINKED.iter().position(|g| *g == group)
}

impl LinkBook {
    pub fn new() -> Self {
        Self::default()
    }

    /// The symbol shared by `group`, or `None` when unset or unlinked.
    pub fn group_symbol(&self, group: LinkGroup) -> Option<&str> {
        slot(group).and_then(|i| self.symbols[i].as_deref())
    }

    /// Sets the symbol for `group`. Blank symbols clear it. Returns `true` when
    /// the stored value changed. Unlinked (`None`) is a no-op.
    pub fn set_group_symbol(&mut self, group: LinkGroup, symbol: Option<&str>) -> bool {
        let Some(i) = slot(group) else { return false };
        let next = symbol.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
        if self.symbols[i] == next {
            return false;
        }
        self.symbols[i] = next;
        true
    }

    /// The group a panel belongs to (`None` if never assigned).
    pub fn panel_group(&self, panel_id: &str) -> LinkGroup {
        self.panels.get(panel_id).copied().unwrap_or(LinkGroup::None)
    }

    /// Assigns a panel to a group. Assigning `None` removes the entry.
    pub fn set_panel_group(&mut self, panel_id: &str, group: LinkGroup) {
        if group == LinkGroup::None {
            self.panels.remove(panel_id);
        } else {
            self.panels.insert(panel_id.to_string(), group);
        }
    }

    /// Panel ids currently in `group`, in id order.
    pub fn panels_in(&self, group: LinkGroup) -> Vec<String> {
        self.panels
            .iter()
            .filter(|(_, g)| **g == group)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

/// Reactive handle to the [`LinkBook`]. `Copy`, so it can be captured freely.
#[derive(Clone, Copy)]
pub struct SymbolLinkContext {
    book: RwSignal<LinkBook>,
}

impl SymbolLinkContext {
    pub fn new() -> Self {
        Self { book: RwSignal::new(LinkBook::new()) }
    }

    /// Makes this context available to descendants via [`use_symbol_link`].
    pub fn provide(self) {
        provide_context(self);
    }

    pub fn group_symbol(&self, group: LinkGroup) -> Option<String> {
        self.book.with(|b| b.group_symbol(group).map(str::to_string))
    }

    pub fn set_group_symbol(&self, group: LinkGroup, symbol: &str) {
        self.book.update(|b| {
            b.set_group_symbol(group, Some(symbol));
        });
    }

    pub fn panel_group(&self, panel_id: &str) -> LinkGroup {
        self.book.with(|b| b.panel_group(panel_id))
    }

    pub fn set_panel_group(&self, panel_id: &str, group: LinkGroup) {
        self.book.update(|b| b.set_panel_group(panel_id, group));
    }
}

impl Default for SymbolLinkContext {
    fn default() -> Self {
        Self::new()
    }
}

/// The context from the nearest provider. Without a provider this returns a
/// fresh, unshared context, so a panel still works standalone (it just does
/// not sync with others).
pub fn use_symbol_link() -> SymbolLinkContext {
    use_context::<SymbolLinkContext>().unwrap_or_default()
}

/// What a panel needs to take part in a link group.
#[derive(Clone, Copy)]
pub struct LinkGroupHandle {
    /// The panel's group (`None` when unlinked).
    pub group: Signal<LinkGroup>,
    /// The shared symbol of the panel's group, or `None` when unlinked/unset.
    pub symbol: Signal<Option<String>>,
    /// Joins or leaves a group (e.g. from the [`HfLinkGroupTag`](honba_kao_ui_core::HfLinkGroupTag)).
    pub set_group: Callback<LinkGroup>,
    /// Publishes a new symbol to every panel in the group. No-op when unlinked.
    pub set_symbol: Callback<String>,
}

/// Wires a panel into the link system. `panel_id` must be stable for the
/// panel's lifetime (for example the route plus a pane slot).
pub fn use_link_group(panel_id: &str) -> LinkGroupHandle {
    let ctx = use_symbol_link();
    let id = panel_id.to_string();

    let group = {
        let id = id.clone();
        Signal::derive(move || ctx.panel_group(&id))
    };
    let symbol = Signal::derive(move || {
        let g = group.get();
        if g == LinkGroup::None {
            None
        } else {
            ctx.group_symbol(g)
        }
    });
    let set_group = {
        let id = id.clone();
        Callback::new(move |g: LinkGroup| ctx.set_panel_group(&id, g))
    };
    let set_symbol = Callback::new(move |sym: String| {
        let g = group.get_untracked();
        if g != LinkGroup::None {
            ctx.set_group_symbol(g, &sym);
        }
    });

    LinkGroupHandle { group, symbol, set_group, set_symbol }
}
