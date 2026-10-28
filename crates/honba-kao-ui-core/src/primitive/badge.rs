//! `HfBadge`, `HfStatusChip`, `HfMoneynessBadge` — status / moneyness pill atoms.
//! `HfMoneynessBadge` renders `ITM`, `ATM`, `OTM`.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Success,
    Warning,
    Danger,
    Info,
}

impl BadgeVariant {
    fn class(self) -> &'static str {
        match self {
            BadgeVariant::Default => "bg-surface-elevated border-border-soft text-text-high",
            BadgeVariant::Success => "bg-emerald-600/20 border-emerald-500/30 text-emerald-400",
            BadgeVariant::Warning => "bg-amber-600/20 border-amber-500/30 text-amber-400",
            BadgeVariant::Danger => "bg-rose-600/20 border-rose-500/30 text-rose-400",
            BadgeVariant::Info => "bg-blue-600/20 border-blue-500/30 text-blue-400",
        }
    }

    fn text_class(self) -> &'static str {
        match self {
            BadgeVariant::Default => "text-text-high",
            BadgeVariant::Success => "text-emerald-400",
            BadgeVariant::Warning => "text-amber-400",
            BadgeVariant::Danger => "text-rose-400",
            BadgeVariant::Info => "text-blue-400",
        }
    }
}

/// Generic status/badge pill.
#[component]
pub fn HfBadge(
    #[prop(into, optional)] variant: Option<BadgeVariant>,
    #[prop(into, optional)] outline: Option<bool>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or_default();
    let outline = outline.unwrap_or(false);
    let base = if outline {
        "border px-1.5 py-0.5 rounded-md text-xs font-semibold border-transparent"
    } else {
        "px-1.5 py-0.5 rounded-md text-xs font-semibold"
    };
    let color = if outline {
        variant.text_class()
    } else {
        variant.class()
    };
    view! {
        <span class=format!("hf-badge {} {}", base, color)>{children()}</span>
    }
}

/// Status chip with a colored dot indicator.
#[component]
pub fn HfStatusChip(
    #[prop(into)] status: String,
    #[prop(into, optional)] variant: Option<BadgeVariant>,
) -> impl IntoView {
    let variant = variant.unwrap_or(BadgeVariant::Default);
    let color = variant.class();
    view! {
        <div class="hf-status-chip inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-semibold border">
            <span class="hf-chip-dot w-1.5 h-1.5 rounded-full"></span>
            <span class=format!("hf-chip-text {}", color.split(' ').last().unwrap_or("text-text-high"))>{status}</span>
        </div>
    }
}

/// Moneyness badge: `ITM`, `ATM`, `OTM`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moneyness {
    ITM,
    ATM,
    OTM,
}

#[component]
pub fn HfMoneynessBadge(
    #[prop(into)] moneyness: Moneyness,
) -> impl IntoView {
    let (label, class) = match moneyness {
        Moneyness::ITM => ("ITM", "bg-emerald-950/20 text-emerald-400 border-emerald-500/30"),
        Moneyness::ATM => ("ATM", "bg-surface-elevated text-text-high border-border-strong font-bold"),
        Moneyness::OTM => ("OTM", "bg-rose-950/20 text-rose-400 border-rose-500/30"),
    };
    view! {
        <span class=format!("hf-moneyness-badge px-1.5 py-0.5 rounded-md text-[10px] font-bold border {}", class)>
            {label}
        </span>
    }
}
