//! Two-way numeric input shared by the order ticket and order modal.
//!
//! React's controlled inputs map to a small sync pair in Leptos: an effect
//! pushes store → display and another commits display → store, converging
//! without loops because each side only writes the signal it does not read.

use leptos::prelude::*;

/// A numeric text input bound to a reactive `f64`.
///
/// `value` is tracked on every change; `on_set` commits parsed user input.
/// While the user is mid-edit their text wins until the next store change,
/// matching the labs' controlled inputs. `placeholder` and `disabled` take
/// reactive signals too, so market-vs-limit gating never remounts the field.
#[component]
pub fn WbNumberInput(
    value: Signal<f64>,
    on_set: Callback<f64>,
    #[prop(into)] placeholder: Signal<String>,
    #[prop(default = "")] class: &'static str,
    #[prop(into)] disabled: Signal<bool>,
    #[prop(default = 1.0)] min: f64,
    #[prop(default = "any")] step: &'static str,
) -> impl IntoView {
    let text = RwSignal::new(trim_number(value.get_untracked()));

    // Store → display (does not fight the binding while identical).
    Effect::new(move |_| {
        let value = value.get();
        if text.get_untracked().parse::<f64>() != Ok(value) {
            text.set(trim_number(value));
        }
    });

    // Display → store on user edits.
    Effect::new(move |_| {
        let parsed = text.get().trim().parse::<f64>();
        if let Ok(parsed) = parsed
            && parsed.min(min) == parsed
            && parsed != value.get_untracked()
        {
            on_set.run(parsed);
        }
    });

    view! {
        <input
            class=format!("wb-input {}", class)
            type="number"
            inputmode="decimal"
            step=step
            min=min
            disabled=disabled
            placeholder=placeholder
            bind:value=text
        />
    }
}

/// `2954.2` not `2954.20000`; quantities stay integral.
fn trim_number(value: f64) -> String {
    if value == value.trunc() && value.abs() < 1e12 {
        format!("{}", value as i64)
    } else {
        format!("{}", value)
    }
}
