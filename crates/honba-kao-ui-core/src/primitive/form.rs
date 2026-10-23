//! HfForm — accessible form layout and field primitives.

use leptos::prelude::*;

#[component]
pub fn HfForm(
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_submit: Option<Callback<web_sys::SubmitEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <form
            class=format!("hf-form {class}").trim().to_string()
            on:submit=move |ev: web_sys::SubmitEvent| {
                ev.prevent_default();
                if let Some(cb) = on_submit {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </form>
    }
}

#[derive(Clone)]
struct FormFieldName(String);

#[component]
pub fn HfField(
    #[prop(into)] name: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    provide_context(FormFieldName(name.clone()));

    view! {
        <div
            data-field=name
            class=format!("hf-form__field {class}").trim().to_string()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfFormItem(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=format!("hf-form__item {class}").trim().to_string()>
            {children()}
        </div>
    }
}

#[component]
pub fn HfFormLabel(
    #[prop(optional, into)] r#for: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let field_for = if r#for.is_empty() {
        use_context::<FormFieldName>().map(|f| f.0)
    } else {
        Some(r#for)
    };

    view! {
        <label
            r#for=field_for
            class=format!("hf-form__label {class}").trim().to_string()
        >
            {children()}
        </label>
    }
}

#[component]
pub fn HfFormControl(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=format!("hf-form__control {class}").trim().to_string()>
            {children()}
        </div>
    }
}

#[component]
pub fn HfFormMessage(
    #[prop(optional)] error: Option<bool>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let error = error.unwrap_or(true);
    let msg_class = format!(
        "hf-form__message {} {class}",
        if error { "hf-form__message--error" } else { "" }
    )
    .trim()
    .to_string();

    view! {
        <p
            role=error.then_some("alert")
            class=msg_class
        >
            {children()}
        </p>
    }
}

#[component]
pub fn HfFormSubmit(
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);

    view! {
        <button
            type="submit"
            disabled=disabled
            class=format!("hf-button hf-button--default hf-form__submit {class}").trim().to_string()
        >
            {children()}
        </button>
    }
}
