//! HfInput and HfTextarea — form input components with bfx-hf-ui-core styling.

use leptos::prelude::*;

/// HfInput — text input with focus states and validation styling.
///
/// ```rust
/// use leptos::prelude::*;
/// use honba_kao_ui_core::HfInput;
///
/// #[component]
/// fn SymbolInput() -> impl IntoView {
///     let symbol = RwSignal::new(String::new());
///     view! {
///         <HfInput
///             placeholder="Enter symbol..."
///             on_input=Callback::new(move |ev| symbol.set(event_target_value(&ev)))
///         />
///     }
/// }
/// ```
#[component]
pub fn HfInput(
    #[prop(optional)] r#type: Option<&'static str>,
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(optional)] value: Option<String>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional)] error: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_input: Option<Callback<web_sys::Event>>,
    #[prop(optional)] on_change: Option<Callback<web_sys::Event>>,
    #[prop(optional)] on_focus: Option<Callback<web_sys::FocusEvent>>,
    #[prop(optional)] on_blur: Option<Callback<web_sys::FocusEvent>>,
) -> impl IntoView {
    let r#type = r#type.unwrap_or("text");
    let disabled = disabled.unwrap_or(false);
    let error = error.unwrap_or(false);

    let input_class = format!(
        "hf-input {} {}",
        if error { "hf-input--error" } else { "" },
        class
    )
    .trim()
    .to_string();

    view! {
        <input
            class=input_class
            r#type=r#type
            placeholder=placeholder
            prop:value=value.unwrap_or_default()
            disabled=disabled
            on:input=move |ev| {
                if let Some(cb) = on_input {
                    cb.run(ev);
                }
            }
            on:change=move |ev| {
                if let Some(cb) = on_change {
                    cb.run(ev);
                }
            }
            on:focus=move |ev| {
                if let Some(cb) = on_focus {
                    cb.run(ev);
                }
            }
            on:blur=move |ev| {
                if let Some(cb) = on_blur {
                    cb.run(ev);
                }
            }
        />
    }
}

/// HfTextarea — multi-line text input.
#[component]
pub fn HfTextarea(
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(optional)] value: Option<String>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional)] rows: Option<u32>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_input: Option<Callback<web_sys::Event>>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    let rows = rows.unwrap_or(4);

    view! {
        <textarea
            class=format!("hf-textarea {class}").trim().to_string()
            placeholder=placeholder
            prop:value=value.unwrap_or_default()
            disabled=disabled
            rows=rows
            on:input=move |ev| {
                if let Some(cb) = on_input {
                    cb.run(ev);
                }
            }
        />
    }
}
