//! `HfButton`, `HfIconButton`, `HfCloseButton`, `HfChip`.
//!
//! Ported from `secured-finance-app/src/components/atoms/Button` (variants primary /
//! secondary / tertiary, sizes xxs … lg) plus tertiary/danger/outline additions
//! required by the trading cockpit.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Ghost,
    Danger,
    Outline,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    Xxs,
    Xs,
    #[default]
    Sm,
    Md,
    Lg,
}

impl ButtonVariant {
    fn base_class(self) -> &'static str {
        match self {
            ButtonVariant::Primary => {
                "border-transparent bg-emerald-600 text-white hover:bg-emerald-700 active:bg-emerald-500"
            }
            ButtonVariant::Secondary => {
                "border-emerald-200/20 bg-transparent text-emerald-400 hover:border-emerald-400 hover:bg-emerald-400/10 active:bg-emerald-400/20"
            }
            ButtonVariant::Tertiary => {
                "border-border-strong bg-transparent text-text-high hover:border-text-muted hover:bg-surface-subtle active:bg-surface-elevated"
            }
            ButtonVariant::Ghost => {
                "border-transparent bg-transparent text-text-muted hover:bg-surface-subtle hover:text-text-high active:bg-surface-elevated"
            }
            ButtonVariant::Danger => {
                "border-transparent bg-rose-600 text-white hover:bg-rose-700 active:bg-rose-500"
            }
            ButtonVariant::Outline => {
                "border-border-strong bg-transparent text-text-high hover:border-text-muted hover:bg-surface-subtle active:bg-surface-elevated"
            }
        }
    }
}

impl ButtonSize {
    fn class(self) -> &'static str {
        match self {
            ButtonSize::Xxs => "rounded-md px-2 py-0.5 text-[11px] leading-tight",
            ButtonSize::Xs => "rounded-md px-2.5 py-1 text-[11px] leading-tight",
            ButtonSize::Sm => "rounded-lg px-3 py-1 text-xs leading-4",
            ButtonSize::Md => "rounded-md px-3.5 py-2.5 text-sm leading-[18px]",
            ButtonSize::Lg => "rounded-xl px-5 py-3 text-base leading-5",
        }
    }
}

fn button_base(disabled: bool) -> &'static str {
    if disabled {
        "inline-flex items-center justify-center font-medium border transition-colors select-none pointer-events-none"
    } else {
        "inline-flex items-center justify-center font-medium border transition-colors select-none cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-emerald-400"
    }
}

/// Primary action button. Renders an `<a>` when `href` is set, otherwise a `<button>`.
#[component]
pub fn HfButton(
    #[prop(into, optional)] href: Option<String>,
    #[prop(into, optional)] size: Option<ButtonSize>,
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(default = false)] full_width: bool,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] aria_label: Option<String>,
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let size = size.unwrap_or_default();
    let variant = variant.unwrap_or_default();
    let label = aria_label.unwrap_or_else(|| "Button".to_string());
    let custom_class = class.unwrap_or_default();
    let disabled = Signal::derive(move || disabled.as_ref().map(|d| d.get()).unwrap_or(false));
    let classes = move || {
        format!(
            "{} {} {} {} {} {}",
            button_base(disabled.get()),
            variant.base_class(),
            size.class(),
            if full_width { "w-full" } else { "w-fit" },
            if disabled.get() {
                "opacity-50 cursor-not-allowed"
            } else {
                ""
            },
            custom_class,
        )
    };

    match href {
        Some(href) => view! {
            <a
                href=href
                target="_blank"
                rel="noopener noreferrer"
                class=classes.clone()
                aria-label=label
            >
                {children()}
            </a>
        }
        .into_any(),
        None => view! {
            <button
                type="button"
                disabled=disabled.get()
                class=classes.clone()
                aria-label=label
                on:click=move |_| {
                    if !disabled.get() {
                        if let Some(cb) = on_click.as_ref() {
                            cb.run(());
                        }
                    }
                }
            >
                {children()}
            </button>
        }
        .into_any(),
    }
}

/// Icon button with a standardized square hit target.
#[component]
pub fn HfIconButton(
    #[prop(into, optional)] size: Option<ButtonSize>,
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    on_click: Option<Callback<()>>,
    #[prop(into)] aria_label: String,
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let size = size.unwrap_or_default();
    let variant = variant.unwrap_or(ButtonVariant::Ghost);
    let custom_class = class.unwrap_or_default();
    let disabled = Signal::derive(move || disabled.as_ref().map(|d| d.get()).unwrap_or(false));
    let pad = match size {
        ButtonSize::Xxs => "p-0.5 rounded-md",
        ButtonSize::Xs => "p-1 rounded-md",
        ButtonSize::Sm => "p-1.5 rounded-lg",
        ButtonSize::Md => "p-2 rounded-md",
        ButtonSize::Lg => "p-3 rounded-xl",
    };
    let classes = move || {
        format!(
            "{} {} {} {} {}",
            button_base(disabled.get()),
            variant.base_class(),
            pad,
            if disabled.get() {
                "opacity-50 cursor-not-allowed"
            } else {
                ""
            },
            custom_class,
        )
    };

    view! {
        <button
            type="button"
            disabled=disabled.get()
            class=classes
            aria-label=aria_label
            on:click=move |_| {
                if !disabled.get() {
                    if let Some(cb) = on_click.as_ref() {
                        cb.run(());
                    }
                }
            }
        >
            {children()}
        </button>
    }
}

/// Standardized close icon button (×).
#[component]
pub fn HfCloseButton(
    on_click: Option<Callback<()>>,
    #[prop(into, optional)] aria_label: Option<String>,
) -> impl IntoView {
    let label = aria_label.unwrap_or_else(|| "Close".to_string());
    view! {
        <HfIconButton
            size=ButtonSize::Sm
            variant=ButtonVariant::Ghost
            aria_label=label
            on_click=on_click
        >
            <span class="text-base leading-none select-none">"✕"</span>
        </HfIconButton>
    }
}

/// Pill-shaped tag button (e.g. for timeframes, filters, quick tags).
#[component]
pub fn HfChip(
    #[prop(default = false)] active: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let classes = if active {
        "inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium border border-emerald-400 bg-emerald-400/10 text-emerald-300"
    } else {
        "inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium border border-border-strong bg-surface-subtle text-text-muted hover:text-text-high hover:border-text-muted"
    };

    view! {
        <button
            type="button"
            class=classes
            on:click=move |_| {
                if let Some(cb) = on_click.as_ref() {
                    cb.run(());
                }
            }
        >
            {children()}
        </button>
    }
}
