//! HfDialog — modal dialog component with bfx-hf-ui-core styling.
//!
//! Mirrors bfx-hf-ui-core `.hfui-dialog` and provides accessible alternatives
//! to raw `<dialog>` or `<div role="dialog">`.

use leptos::prelude::*;

/// Dialog variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfDialogVariant {
    Default,
    Warning,
    Confirm,
}

impl HfDialogVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            HfDialogVariant::Default => "default",
            HfDialogVariant::Warning => "warning",
            HfDialogVariant::Confirm => "confirm",
        }
    }
}

/// HfDialog — modal dialog shell.
///
/// Provides the overlay and content area. Action buttons (confirm/cancel)
/// are provided via `HfDialogAction` or composed in `HfConfirmDialog`.
#[component]
pub fn HfDialog(
    #[prop(optional)] variant: Option<HfDialogVariant>,
    #[prop(optional)] class: String,
    #[prop(optional, into)] overlay_class: String,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(HfDialogVariant::Default);
    let overlay_variant_class = match variant {
        HfDialogVariant::Default => "hf-dialog--overlay--default",
        HfDialogVariant::Warning => "hf-dialog--overlay--warning",
        HfDialogVariant::Confirm => "hf-dialog--overlay--confirm",
    };

    view! {
        <div
            class=format!("hf-dialog {overlay_variant_class} {class}").trim().to_string()
        >
            <div
                class=format!("hf-dialog__overlay {overlay_class}").trim().to_string()
                aria-hidden="true"
            />
            <div class="hf-dialog__content">{children()}</div>
        </div>
    }
}

/// HfDialogTrigger — button that opens the dialog.
#[component]
pub fn HfDialogTrigger(
    #[prop(optional)] class: String,
    #[prop(optional, into)] title: String,
    #[prop(optional)] variant: Option<HfDialogVariant>,
    on_click: Callback<web_sys::MouseEvent>,
    children: Children,
) -> impl IntoView {
    let _variant = variant.unwrap_or(HfDialogVariant::Default);

    view! {
        <button
            class=format!("hf-dialog-trigger {class}").trim().to_string()
            on:click=move |ev| {
                use wasm_bindgen::JsCast;
                let is_disabled = ev
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::HtmlButtonElement>().ok())
                    .map(|button| button.disabled())
                    .unwrap_or(false);
                if !is_disabled {
                    on_click.run(ev);
                }
            }
        >
            {title}
            {children()}
        </button>
    }
}

/// HfDialogContent — accessible dialog surface (`role="dialog"`, `aria-modal="true"`).
#[component]
pub fn HfDialogContent(
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] aria_labelledby: String,
    children: Children,
) -> impl IntoView {
    let aria_labelledby = (!aria_labelledby.is_empty()).then_some(aria_labelledby);

    view! {
        <div
            class=format!("hf-dialog__body {class}").trim().to_string()
            role="dialog"
            aria-modal="true"
            aria-labelledby=aria_labelledby
        >
            {children()}
        </div>
    }
}

/// HfDialogHeader — top title/description region of a dialog.
#[component]
pub fn HfDialogHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <header class=format!("hf-dialog__header {class}").trim().to_string()>
            {children()}
        </header>
    }
}

/// HfDialogFooter — bottom action bar of a dialog.
#[component]
pub fn HfDialogFooter(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <footer class=format!("hf-dialog__footer {class}").trim().to_string()>
            {children()}
        </footer>
    }
}

/// HfDialogAction — action button within a dialog.
#[component]
pub fn HfDialogAction(
    #[prop(optional)] variant: Option<HfDialogVariant>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(HfDialogVariant::Default);
    let action_class = match variant {
        HfDialogVariant::Default => "hf-dialog-action--default",
        HfDialogVariant::Warning => "hf-dialog-action--warning",
        HfDialogVariant::Confirm => "hf-dialog-action--confirm",
    };

    view! {
        <button
            class=format!("hf-dialog-action {action_class} {class}").trim().to_string()
        >
            {children()}
        </button>
    }
}
