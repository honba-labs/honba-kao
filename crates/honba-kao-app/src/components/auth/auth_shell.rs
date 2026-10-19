//! Shared auth layout: brand panel + form card.

use leptos::prelude::*;

use crate::components::common::BrandLogo;
use crate::constants::branding;

#[component]
pub fn AuthShell(
    title: &'static str,
    #[prop(default = "")] subtitle: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="auth-layout">
            <aside class="auth-layout__brand">
                <div class="auth-brand__mark">
                    <BrandLogo size=44 />
                    <span class="auth-brand__name">{branding::APP_NAME}</span>
                </div>

                <span class="auth-brand__eyebrow">
                    <span class="auth-brand__dot" />
                    "Your Indian markets workspace"
                </span>

                <h1 class="auth-brand__title">
                    "AI agents for "
                    <em>"NSE & BSE"</em>
                </h1>

                <p class="auth-brand__copy">
                    "Connect Zerodha, Dhan, Upstox, Angel One, Fyers and Groww. \
                     Deploy risk-gated agent strategies around the clock — \
                     described in plain language."
                </p>

                <dl class="auth-brand__stats">
                    <div>
                        <dt>"6"</dt>
                        <dd>"Brokers"</dd>
                    </div>
                    <div>
                        <dt>"24/7"</dt>
                        <dd>"Agents on watch"</dd>
                    </div>
                    <div>
                        <dt>"100%"</dt>
                        <dd>"Orders risk-checked"</dd>
                    </div>
                </dl>
            </aside>

            <section class="auth-layout__form">
                <div class="auth-card">
                    <h2 class="auth-card__title">{title}</h2>
                    {(!subtitle.is_empty()).then(|| view! {
                        <p class="auth-card__sub">{subtitle}</p>
                    })}
                    {children()}
                </div>
            </section>
        </div>
    }
}

#[component]
pub fn AuthField(
    label: &'static str,
    /// Two-way bound value. The field is controlled, so the page owns the
    /// state and the submit handler can read it without touching the DOM.
    value: RwSignal<String>,
    #[prop(default = "text")] input_type: &'static str,
    #[prop(default = "")] placeholder: &'static str,
    #[prop(default = "")] autocomplete: &'static str,
    #[prop(default = false)] autofocus: bool,
) -> impl IntoView {
    view! {
        <label class="auth-field">
            <span>{label}</span>
            <input
                type=input_type
                placeholder=placeholder
                autocomplete=autocomplete
                autofocus=autofocus
                prop:value=move || value.get()
                on:input=move |event| value.set(event_target_value(&event))
            />
        </label>
    }
}
