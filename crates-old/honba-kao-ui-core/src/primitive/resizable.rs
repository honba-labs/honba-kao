//! HfResizablePanelGroup / HfResizablePanel / HfResizableHandle — split pane layout primitives.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfResizableDirection {
    Horizontal,
    Vertical,
}

impl HfResizableDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            HfResizableDirection::Horizontal => "horizontal",
            HfResizableDirection::Vertical => "vertical",
        }
    }
}

#[component]
pub fn HfResizablePanelGroup(
    #[prop(optional)] direction: Option<HfResizableDirection>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let direction = direction.unwrap_or(HfResizableDirection::Horizontal);
    let dir_str = direction.as_str();

    view! {
        <div
            data-panel-group-direction=dir_str
            class=format!("hf-resizable-group hf-resizable-group--{dir_str} {class}").trim().to_string()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfResizablePanel(
    #[prop(optional)] default_size: Option<f64>,
    #[prop(optional)] min_size: Option<f64>,
    #[prop(optional)] max_size: Option<f64>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let flex_style = default_size.map(|pct| format!("flex: {pct:.2} 1 0%;"));

    view! {
        <div
            style=flex_style
            data-min-size=min_size.map(|v| v.to_string())
            data-max-size=max_size.map(|v| v.to_string())
            class=format!("hf-resizable-panel {class}").trim().to_string()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfResizableHandle(
    #[prop(optional)] with_handle: Option<bool>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let with_handle = with_handle.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);

    view! {
        <div
            role="separator"
            tabindex=if disabled { "-1" } else { "0" }
            aria-disabled=disabled.to_string()
            class=format!(
                "hf-resizable-handle {} {class}",
                if disabled { "hf-resizable-handle--disabled" } else { "" }
            )
            .trim()
            .to_string()
        >
            {with_handle.then(|| view! { <div class="hf-resizable-handle__grip" /> })}
        </div>
    }
}
