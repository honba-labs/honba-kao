//! `HfSparklinePrimitive` — lightweight inline sparkline for telemetry feeds.

use leptos::prelude::*;

#[component]
pub fn HfSparklinePrimitive(
    #[prop(into)] values: Signal<Vec<f64>>,
    #[prop(into, optional)] height: Option<&'static str>,
    #[prop(into, optional)] color: Option<&'static str>,
) -> impl IntoView {
    let height = height.unwrap_or("h-10");
    let color = color.unwrap_or("text-emerald-400");
    let (min, max) = Signal::derive(move || {
        let vals = values.get();
        if vals.is_empty() {
            (0.0, 1.0)
        } else {
            let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
            let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mx = if (mx - mn).abs() < f64::EPSILON { mn + 1.0 } else { mx };
            (mn, mx)
        }
    });

    view! {
        <svg
            class=format!("hf-sparkline {} {}", height, color)
            view_box="0 0 100 40"
            preserve_aspect_ratio="none"
        >
            <polyline
                class="fill-none stroke-current stroke-1.5"
                points=move || {
                    let vals = values.get();
                    let (mn, mx) = (min.get(), max.get());
                    let n = vals.len();
                    if n == 0 {
                        String::new()
                    } else {
                        vals.iter()
                            .enumerate()
                            .map(|(i, v)| {
                                let x = if n == 1 { 50.0 } else { i as f64 / (n - 1) as f64 * 100.0 };
                                let y = 40.0 - ((v - mn) / (mx - mn) * 40.0);
                                format!("{x:.1},{y:.1}")
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    }
                }
            />
        </svg>
    }
}
