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
            ButtonVariant::Primary => "border-transparent bg-emerald-600 text-white hover:bg-emerald-700 active:bg-emerald-500",
            ButtonVariant::Secondary => "border-emerald-200/20 bg-transparent text-emerald-400 hover:border-emerald-400 hover:bg-emerald-400/10 active:bg-emerald-400/20",
            ButtonVariant::Tertiary => "border-border-strong bg-transparent text-text-high hover:border-text-muted hover:bg-surface-subtle active:bg-surface-elevated",
            ButtonVariant::Ghost => "border-transparent bg-transparent text-text-muted hover:bg-surface-subtle hover:text-text-high active:bg-surface-elevated",
            ButtonVariant::Danger => "border-transparent bg-rose-600 text-white hover:bg-rose-700 active:bg-rose-500",
            ButtonVariant::Outline => "border-border-strong bg-transparent text-text-high hover:border-text-muted hover:bg-surface-subtle active:bg-surface-elevated",
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

fn button_base(disabled: bool) -> String {
    let mut classes = vec![
        "flex items-center justify-center border font-semibold whitespace-nowrap",
    ];
    if disabled {
        classes.push("opacity-50 cursor-not-allowed");
    }
    classes.join(" ")
}

/// Primary action button. Renders an `<a>` when `href` is set, otherwise a `<button>`.
#[component]
pub fn HfButton(
    #[prop(into, optional)] href: Option<String>,
    #[prop(into, optional)] size: Option<ButtonSize>,
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(default = false)] full_width: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] aria_label: Option<String>,
    children: Children,
) -> impl IntoView {
    let size = size.unwrap_or_default();
    let variant = variant.unwrap_or_default();
    let label = aria_label.unwrap_or_else(|| "Button".to_string());

    let classes = format!(
        "{} {} {} {}",
        button_base(disabled),
        variant.base_class(),
        size.class(),
        if full_width { "w-full" } else { "w-fit" }
    );

    match href {
        Some(href) => view! {
            <a
                href=href
                target="_blank"
                rel="noopener noreferrer"
                class=classes
                aria-label=label
            >
                {children()}
            </a>
        }
        .into_view(),
        None => view! {
            <button
                type="button"
                disabled=disabled
                class=classes
                aria-label=label
                on:click=move |_| {
                    if let Some(cb) = on_click.as_ref() {
                        cb.call(());
                    }
                }
            >
                {children()}
            </button>
        }
        .into_view(),
    }
}

/// Icon-only button (toolbar / modal controls).
#[component]
pub fn HfIconButton(
    #[prop(into, optional)] size: Option<ButtonSize>,
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(default = false)] disabled: bool,
    #[prop(into)] on_click: Callback<()>,
    #[prop(into, optional)] aria_label: Option<String>,
    children: Children,
) -> impl IntoView {
    let size = size.unwrap_or(ButtonSize::Sm);
    let variant = variant.unwrap_or(ButtonVariant::Ghost);
    let label = aria_label.unwrap_or_else(|| "Button".to_string());
    let classes = format!(
        "{} {} {}",
        button_base(disabled),
        variant.base_class(),
        size.class()
    );
    view! {
        <button
            type="button"
            disabled=disabled
            class=classes
            aria-label=label
            on:click=move |_| on_click.call(())
        >
            {children()}
        </button>
    }
}

/// High-contrast dismiss button for modals, notifications, toolbars.
#[component]
pub fn HfCloseButton(
    #[prop(into)] on_click: Callback<()>,
    #[prop(into, optional)] aria_label: Option<String>,
) -> impl IntoView {
    let label = aria_label.unwrap_or_else(|| "Close".to_string());
    view! {
        <HfIconButton
            size=ButtonSize::Xs
            variant=ButtonVariant::Ghost
            on_click=on_click
            aria_label=label
        >
            <span class="hf-icon hf-icon-close" aria_hidden="true">"✕"</span>
        </HfIconButton>
    }
}

/// Compact interactive pill tag for filtering, status, or tag selection.
#[component]
pub fn HfChip(
    #[prop(into, optional)] selected: Option<bool>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let selected = selected.unwrap_or(false);
    let classes = format!(
        "hf-chip inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-xs font-medium border transition-colors {}",
        if selected {
            "bg-emerald-600/20 border-emerald-500 text-emerald-300"
        } else {
            "bg-surface-subtle border-border-soft text-text-muted hover:bg-surface-elevated hover:text-text-high"
        }
    );
    view! {
        <button
            type="button"
            class=classes
            on:click=move |_| {
                if let Some(cb) = on_click.as_ref() {
                    cb.call(());
                }
            }
        >
            {children()}
        </button>
    }
}
