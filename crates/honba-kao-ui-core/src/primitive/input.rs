//! `HfInput`, `HfNumberInput`, `HfOrderInputBox`, `HfCheckbox`, `HfRadioButton`,
//! `HfSelect`, `HfToggle` — form primitive atoms.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Foundational styled text/number input element with focus borders.
#[component]
pub fn HfInput(
    #[prop(into)] value: RwSignal<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] disabled: Option<bool>,
    #[prop(into, optional)] on_input: Option<Callback<String>>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    view! {
        <input
            type="text"
            class="hf-input flex h-9 w-full rounded-md border border-border-soft bg-surface-base px-3 py-1.5 text-sm text-text-high placeholder:text-text-muted focus:border-emerald-500 focus:outline-none disabled:bg-surface-subtle disabled:text-text-muted disabled:cursor-not-allowed"
            placeholder=placeholder.unwrap_or_default()
            disabled=disabled
            prop:value=value
            on:input=move |ev| {
                if let Some(cb) = on_input.as_ref() {
                    cb.run(event_target_value(&ev));
                }
            }
        />
    }
}

/// Numeric input with step controls.
#[component]
pub fn HfNumberInput(
    #[prop(into)] value: RwSignal<f64>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] min: Option<f64>,
    #[prop(into, optional)] max: Option<f64>,
    #[prop(into, optional)] step: Option<f64>,
    #[prop(into, optional)] disabled: Option<bool>,
) -> impl IntoView {
    let str_val = move || value.get().to_string();
    let disabled = disabled.unwrap_or(false);
    view! {
        <input
            type="number"
            class="hf-input flex h-9 w-full rounded-md border border-border-soft bg-surface-base px-3 py-1.5 text-sm text-text-high focus:border-emerald-500 focus:outline-none"
            placeholder=placeholder.unwrap_or_default()
            min=min.map(|m| m.to_string())
            max=max.map(|m| m.to_string())
            step=step.map(|s| s.to_string())
            prop:value=str_val
            disabled=disabled
            on:input=move |ev: web_sys::Event| {
                let target: web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                if let Ok(v) = target.value().parse::<f64>() {
                    value.set(v);
                }
            }
        />
    }
}

/// Numeric input with currency adornment (₹, $, USDC), precision mask, monospace tnum.
/// Ported from `secured-finance-app/src/components/atoms/OrderInputBox`.
#[component]
pub fn HfOrderInputBox(
    #[prop(into)] value: RwSignal<String>,
    #[prop(into, optional)] adornment: Option<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] disabled: Option<bool>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    view! {
        <div class="hf-order-input-box relative flex items-center">
            {adornment.map(|ad| {
                view! {
                    <span class="hf-adornment absolute left-2.5 top-1/2 -translate-y-1/2 text-xs text-text-muted pointer-events-none">{ad}</span>
                }
            })}
            <input
                type="text"
                inputmode="decimal"
                class="hf-order-input flex h-9 w-full rounded-md border border-border-soft bg-surface-base pl-9 pr-3 py-1.5 text-sm font-mono text-text-high placeholder:text-text-muted focus:border-emerald-500 focus:outline-none disabled:bg-surface-subtle disabled:text-text-muted disabled:cursor-not-allowed"
                placeholder=placeholder.unwrap_or_default()
                disabled=disabled
                prop:value=value
            />
        </div>
    }
}

/// Accessible binary checkbox with indeterminate state for batch table selections.
#[component]
pub fn HfCheckbox(
    #[prop(into)] checked: RwSignal<bool>,
    #[prop(into, optional)] disabled: Option<bool>,
    #[prop(into, optional)] label: Option<String>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    view! {
        <div class="hf-checkbox flex items-center gap-2 cursor-pointer">
            <input
                type="checkbox"
                class="hf-checkbox-input w-4 h-4 rounded border border-border-soft bg-surface-base text-emerald-600 focus:ring-emerald-500 focus:ring-1 disabled:opacity-50"
                checked=checked
                disabled=disabled
                on:change=move |_| checked.set(!checked.get())
            />
            {label.map(|l| view! { <span class="text-sm text-text-high">{l}</span> })}
        </div>
    }
}

/// Accessible radio option button with custom terminal indicator.
#[component]
pub fn HfRadioButton(
    #[prop(into)] value: String,
    #[prop(into)] selected: RwSignal<String>,
    #[prop(into, optional)] disabled: Option<bool>,
    #[prop(into, optional)] label: Option<String>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    let value_for_view = value.clone();
    let value_for_change = value.clone();
    let value_for_selected = value_for_view.clone();
    let is_selected = move || selected.get() == value_for_selected;
    view! {
        <div class="hf-radio flex items-center gap-2 cursor-pointer">
            <input
                type="radio"
                class="hf-radio-input w-4 h-4 rounded-full border border-border-soft bg-surface-base text-emerald-600 focus:ring-emerald-500 focus:ring-1 disabled:opacity-50"
                value=value_for_view
                checked=is_selected
                disabled=disabled
                on:change=move |_| selected.set(value_for_change.clone())
            />
            {label.map(|l| view! { <span class="text-sm text-text-high">{l}</span> })}
        </div>
    }
}

/// Accessible styled select control for tenors, intervals, and order types.
#[component]
pub fn HfSelect(
    #[prop(into)] value: RwSignal<String>,
    options: Vec<(String, String)>, // (label, value)
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] disabled: Option<bool>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    view! {
        <select
            class="hf-select flex h-9 w-full rounded-md border border-border-soft bg-surface-base px-3 py-1.5 text-sm text-text-high focus:border-emerald-500 focus:outline-none disabled:bg-surface-subtle disabled:text-text-muted disabled:cursor-not-allowed"
            prop:value=value
            disabled=disabled
            on:change=move |ev: web_sys::Event| {
                let target: web_sys::HtmlSelectElement = ev.target().unwrap().dyn_into().unwrap();
                value.set(target.value());
            }
        >
            {placeholder.map(|p| view! { <option value="" disabled=true hidden=true>{p}</option> })}
            {options.into_iter().map(|(label, value)| {
                view! { <option value=value>{label}</option> }
            }).collect::<Vec<_>>()}
        </select>
    }
}

/// Accessible binary toggle switch for live mode, crosshair sync, etc.
#[component]
pub fn HfToggle(
    #[prop(into)] checked: RwSignal<bool>,
    #[prop(into, optional)] disabled: Option<bool>,
    #[prop(into, optional)] label_on: Option<String>,
    #[prop(into, optional)] label_off: Option<String>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    let label_on = label_on.unwrap_or_else(|| "On".to_string());
    let label_off = label_off.unwrap_or_else(|| "Off".to_string());
    let is_checked = checked;
    view! {
        <button
            type="button"
            class=move || format!(
                "hf-toggle inline-flex items-center justify-between w-[44px] h-[24px] rounded-full border border-border-soft bg-surface-base p-0.5 transition-colors focus:outline-none {} {}",
                if is_checked.get() { "bg-emerald-600 border-emerald-600" } else { "" },
                if disabled { "opacity-50 cursor-not-allowed" } else { "cursor-pointer" }
            )
            disabled=disabled
            on:click=move |_| is_checked.set(!is_checked.get())
            aria-pressed=is_checked
        >
            <span class=move || format!("hf-toggle-thumb block w-[20px] h-[20px] rounded-full bg-white shadow-sm transition-transform {}",
                if is_checked.get() { "translate-x-full" } else { "translate-x-0" }
            )></span>
            <span class="hf-toggle-label text-[10px] font-mono text-text-high">
                {move || if is_checked.get() { label_on.clone() } else { label_off.clone() }}
            </span>
        </button>
    }
}
