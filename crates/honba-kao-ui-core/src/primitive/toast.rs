//! HfToast — toast notification component with bfx-hf-ui-core styling.
//!
//! Mirrors bfx-hf-ui-core `.hfui-toast` and provides a toast provider/viewport
//! system for transient success/error messages.

use leptos::prelude::*;

/// Toast variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfToastVariant {
    Success,
    Error,
    Info,
}

impl HfToastVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            HfToastVariant::Success => "success",
            HfToastVariant::Error => "error",
            HfToastVariant::Info => "info",
        }
    }
}

/// Toast action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfToastAction {
   Dismiss,
   Confirm,
}

/// HfToast — individual toast notification.
#[component]
pub fn HfToast(
    #[prop(optional)]
    variant: Option<HfToastVariant>,
    #[prop(optional, into)]
    title: String,
    #[prop(optional, into)]
    description: String,
    #[prop(optional)]
    action: Option<HfToastAction>,
    #[prop(optional)]
    duration: Option<u32>,
) -> impl IntoView {
    let variant = variant.unwrap_or(HfToastVariant::Info);
    let action_str = match action {
        Some(HfToastAction::Dismiss) => "dismiss",
        Some(HfToastAction::Confirm) => "confirm",
        None => "",
    };
    let dur = duration.unwrap_or(5000);

    let classes = format!(
        "hf-toast hf-toast--{}",
        variant.as_str()
    );

    view! {
        <div
            class=classes
            role="alert"
            aria-live="polite"
            aria-atomic="true"
        >
            <div class="hf-toast__icon" />
            <div class="hf-toast__content">
                <strong class="hf-toast__title">{title}</strong>
                <p class="hf-toast__description">{description}</p>
            </div>
            <button
                class="hf-toast__action"
                on:click=move |_| {
                    // dispatch dismiss event or remove from viewport
                }
            >
                {action_str}
            </button>
        </div>
    }
}

/// HfToastProvider — manages toast viewport and stacking.
#[component]
pub fn HfToastProvider(
    /// Children rendered inside the toast viewport.
    children: Children,
    /// Maximum number of concurrent toasts.
    #[prop(optional)]
    max_toasts: Option<usize>,
) -> impl IntoView {
    view! {
        <div class="hf-toast-viewport" data-max-toasts=max_toasts.map(|max| max.to_string())>
            {children()}
        </div>
    }
}

/// HfToastViewport — the area where toasts are rendered.
#[component]
pub fn HfToastViewport() -> impl IntoView {
    view! {
        <div class="hf-toast-viewport__inner" />
    }
}
