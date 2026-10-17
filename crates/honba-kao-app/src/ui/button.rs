//! Button primitives modeled after bfx-hf-ui-core .hfui-button.

use leptos::prelude::*;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Default,
    Green,
    Red,
    Blue,
    Ghost,
}

impl ButtonVariant {
    pub fn as_class(&self) -> &'static str {
        match self {
            ButtonVariant::Default => "",
            ButtonVariant::Green => "green",
            ButtonVariant::Red => "red",
            ButtonVariant::Blue => "blue",
            ButtonVariant::Ghost => "ghost",
        }
    }
}

#[component]
pub fn HfButton(
    #[prop(default = ButtonVariant::Default)] variant: ButtonVariant,
    #[prop(default = false)] small: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(default = "")] class: &'static str,
    children: Children,
) -> impl IntoView {
    let variant_class = variant.as_class();
    let size_class = if small { "small" } else { "" };

    let handle_click = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        if !disabled {
            if let Some(cb) = on_click {
                cb.run(());
            }
        }
    };

    view! {
        <button
            class=format!("hfui-button {} {} {}", variant_class, size_class, class)
            disabled=disabled
            on:click=handle_click
        >
            {children()}
        </button>
    }
}
