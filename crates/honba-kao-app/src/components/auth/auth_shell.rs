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
                <BrandLogo size=44 />
                <h1 class="auth-layout__brand-name">{branding::APP_NAME}</h1>
                <p class="auth-layout__tagline">{branding::TAGLINE}</p>
                <ul class="auth-layout__points">
                    <li>"Risk-gated execution on NSE / BSE"</li>
                    <li>"Paper trading by default"</li>
                    <li>"Full decision audit trail"</li>
                </ul>
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
    #[prop(default = "text")] input_type: &'static str,
    #[prop(default = "")] placeholder: &'static str,
) -> impl IntoView {
    view! {
        <label class="auth-field">
            <span>{label}</span>
            <input type=input_type placeholder=placeholder />
        </label>
    }
}
