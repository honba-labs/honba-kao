//! HfButton and HfIconButton — variants: Default/Green/Red/Blue/Ghost
//!
//! Mirrors bfx-hf-ui-core `.hfui-button` styling language. Styling lives in
//! the app's `style/main.css` (token-driven, themed via `data-theme`); these
//! components only emit the `hf-button`/`hf-icon-button` class contract plus
//! `aria-pressed`/`aria-expanded` for toggle/menu-trigger state, so callers
//! get real-time reactivity without re-deriving a class string per call site.

use leptos::prelude::*;

/// Button variant mapping to bfx-hf-ui-core color tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfButtonVariant {
    /// Default — `--surface` bg, `--ink-2` text
    Default,
    /// Green — `--button-green` bg, `--on-accent` text
    Green,
    /// Red — `--button-red` bg, `--on-accent` text
    Red,
    /// Blue — `--button-blue` bg, `--on-accent` text
    Blue,
    /// Ghost — transparent bg, `--shade-6` text
    Ghost,
}

impl HfButtonVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            HfButtonVariant::Default => "default",
            HfButtonVariant::Green => "green",
            HfButtonVariant::Red => "red",
            HfButtonVariant::Blue => "blue",
            HfButtonVariant::Ghost => "ghost",
        }
    }
}

/// Button size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfButtonSize {
    Small,
    Default,
    Large,
}

impl HfButtonSize {
    pub fn as_str(self) -> &'static str {
        match self {
            HfButtonSize::Small => "sm",
            HfButtonSize::Default => "default",
            HfButtonSize::Large => "lg",
        }
    }
}

/// HfButton — primary action button component.
///
/// `active` and `disabled` take anything `Into<Signal<bool>>` (a plain
/// `bool`, a `Signal`/`Memo`/`RwSignal`, or `Signal::derive(move || ...)`),
/// so toggle/pill/segmented usages stay reactive without rebuilding the
/// button on every tick.
///
/// ```rust
/// view! {
///     <HfButton variant=HfButtonVariant::Green on_click=|_| {}>
///         "Buy"
///     </HfButton>
/// }
/// ```
#[component]
pub fn HfButton(
    #[prop(optional)]
    variant: Option<HfButtonVariant>,
    #[prop(optional)]
    size: Option<HfButtonSize>,
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Toggle/pressed state — reflected as `aria-pressed` and styled via the
    /// `.hf-button[aria-pressed="true"]` rule, no extra class bookkeeping.
    #[prop(optional, into)]
    active: Signal<bool>,
    #[prop(optional)]
    loading: Option<bool>,
    #[prop(optional)]
    on_click: Option<Callback<web_sys::MouseEvent>>,
    #[prop(optional, into)]
    class: String,
    #[prop(optional, into)]
    title: String,
    /// ARIA role passthrough, e.g. `"menuitem"` / `"menuitemradio"` for items
    /// inside a dropdown menu.
    #[prop(optional, into)]
    role: String,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(HfButtonVariant::Default);
    let size = size.unwrap_or(HfButtonSize::Default);
    let loading = loading.unwrap_or(false);

    let variant_class = match variant {
        HfButtonVariant::Default => "hf-button--default",
        HfButtonVariant::Green => "hf-button--green",
        HfButtonVariant::Red => "hf-button--red",
        HfButtonVariant::Blue => "hf-button--blue",
        HfButtonVariant::Ghost => "hf-button--ghost",
    };

    let size_class = match size {
        HfButtonSize::Small => "hf-button--sm",
        HfButtonSize::Default => "hf-button--default-size",
        HfButtonSize::Large => "hf-button--lg",
    };

    let full_class = move || {
        let mut classes = vec!["hf-button", variant_class, size_class];
        if loading {
            classes.push("hf-button--loading");
        }
        if !class.is_empty() {
            classes.push(&class);
        }
        classes.join(" ")
    };

    let content = if loading {
        view! { <span class="hf-button__loader" /> }.into_any()
    } else {
        children().into_any()
    };

    let role = (!role.is_empty()).then_some(role);

    view! {
        <button
            class=full_class
            disabled=move || disabled.get()
            aria-pressed=move || active.get().to_string()
            title=title
            role=role
            on:click=move |ev| {
                if !disabled.get() && !loading {
                    if let Some(cb) = on_click {
                        cb.run(ev);
                    }
                }
            }
        >
            {content}
        </button>
    }
}

/// HfIconButton — icon-only button for toolbars and secondary actions.
///
/// ```rust
/// use honba_kao_ui_core::icons::outline::settings;
///
/// view! {
///     <HfIconButton icon=settings() size=16 />
/// }
/// ```
#[component]
pub fn HfIconButton(
    /// The SVG path `d` data (from any `outline::*` function). Accepts a
    /// plain `&'static str` or `Signal::derive(...)` for icons that swap
    /// based on state (e.g. a light/dark theme toggle).
    #[prop(into)]
    icon: Signal<&'static str>,
    #[prop(optional)]
    size: Option<u32>,
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Pressed/open state — reflected as `aria-pressed`.
    #[prop(optional, into)]
    active: Signal<bool>,
    /// Set when this button opens a menu/popover (`aria-haspopup` + `aria-expanded`).
    #[prop(optional, into)]
    aria_expanded: Option<Signal<bool>>,
    #[prop(optional)]
    on_click: Option<Callback<web_sys::MouseEvent>>,
    #[prop(optional, into)]
    class: String,
    #[prop(optional, into)]
    title: String,
) -> impl IntoView {
    let size_px = size.unwrap_or(20);
    let size_class = match size {
        Some(16) => "hf-icon-button--sm",
        Some(24) => "hf-icon-button--lg",
        _ => "hf-icon-button--default",
    };

    let full_class = format!("hf-icon-button {size_class} {class}").trim().to_string();
    let has_popup = aria_expanded.is_some();
    let aria_label = title.clone();

    view! {
        <button
            class=full_class
            disabled=move || disabled.get()
            title=title
            aria-label=aria_label
            aria-pressed=move || active.get().to_string()
            aria-haspopup=move || if has_popup { "menu" } else { "false" }
            aria-expanded=move || aria_expanded.map(|signal| signal.get()).unwrap_or(false).to_string()
            on:click=move |ev| {
                if !disabled.get() {
                    if let Some(cb) = on_click {
                        cb.run(ev);
                    }
                }
            }
        >
            <svg
                viewBox="0 0 24 24"
                stroke="currentColor"
                fill="none"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="hf-icon-button__icon"
                width=size_px.to_string()
                height=size_px.to_string()
            >
                <path d=move || icon.get() />
            </svg>
        </button>
    }
}
