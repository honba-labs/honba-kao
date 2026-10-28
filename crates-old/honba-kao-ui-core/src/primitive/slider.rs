//! HfSlider — accessible range slider primitive (`role="slider"` via `<input type="range">`).

use leptos::prelude::*;

#[component]
pub fn HfSlider(
    #[prop(optional)] value: Option<f64>,
    #[prop(optional)] min: Option<f64>,
    #[prop(optional)] max: Option<f64>,
    #[prop(optional)] step: Option<f64>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] aria_label: String,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_change: Option<Callback<f64>>,
) -> impl IntoView {
    let min = min.unwrap_or(0.0);
    let max = max.unwrap_or(100.0);
    let step = step.unwrap_or(1.0);
    let value = value.unwrap_or(min).clamp(min, max);
    let disabled = disabled.unwrap_or(false);
    let aria_label = (!aria_label.is_empty()).then_some(aria_label);

    view! {
        <input
            type="range"
            class=format!("hf-slider {class}").trim().to_string()
            min=min.to_string()
            max=max.to_string()
            step=step.to_string()
            prop:value=value.to_string()
            disabled=disabled
            aria-label=aria_label
            aria-valuemin=min.to_string()
            aria-valuemax=max.to_string()
            aria-valuenow=value.to_string()
            on:input=move |ev| {
                if let Some(cb) = on_change
                    && let Ok(parsed) = event_target_value(&ev).parse::<f64>() {
                        cb.run(parsed);
                    }
            }
        />
    }
}
