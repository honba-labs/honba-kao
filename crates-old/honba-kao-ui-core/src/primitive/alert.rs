//! HfAlert — inline status callout (`role="alert"`).

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfAlertVariant {
    Default,
    Info,
    Warning,
    Destructive,
}

impl HfAlertVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            HfAlertVariant::Default => "default",
            HfAlertVariant::Info => "info",
            HfAlertVariant::Warning => "warning",
            HfAlertVariant::Destructive => "destructive",
        }
    }
}

#[component]
pub fn HfAlert(
    #[prop(optional)] variant: Option<HfAlertVariant>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(HfAlertVariant::Default);

    view! {
        <div
            role="alert"
            class=format!("hf-alert hf-alert--{} {class}", variant.as_str()).trim().to_string()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfAlertTitle(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <h5 class=format!("hf-alert__title {class}").trim().to_string()>
            {children()}
        </h5>
    }
}

#[component]
pub fn HfAlertDescription(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("hf-alert__description {class}").trim().to_string()>
            {children()}
        </div>
    }
}
