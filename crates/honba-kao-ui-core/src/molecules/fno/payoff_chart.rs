//! `HfOptionPayoffChart` — multi-leg payoff diagram at maturity.
//! Compute lives in `honba-kao-api-wasm`; this is the UI molecule only.

use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PayoffPoint {
    pub spot: f64,
    pub pnl: f64,
}

#[component]
pub fn HfOptionPayoffChart(
    points: Signal<Vec<PayoffPoint>>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let base = "hf-payoff-chart w-full h-48 rounded border border-border-soft bg-surface-base";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    let range = Signal::derive(move || {
        let pts = points.get();
        if pts.is_empty() { (0.0, 0.0) } else {
            let mn = pts.iter().map(|p| p.pnl).fold(f64::INFINITY, f64::min);
            let mx = pts.iter().map(|p| p.pnl).fold(f64::NEG_INFINITY, f64::max);
            let mx = if (mx - mn).abs() < f64::EPSILON { mn + 1.0 } else { mx };
            (mn, mx)
        }
    });
    view! {
        <div class=classes>
            <svg class="hf-payoff-svg w-full h-full" view_box="0 0 100 50" preserve_aspect_ratio="none">
                <polyline class="fill-none stroke-emerald-500 stroke-1.5"
                    points=move || {
                        let pts = points.get();
                        let (mn, mx) = (range.get().0, range.get().1);
                        if pts.is_empty() { String::new() } else {
                            pts.iter().enumerate().map(|(i, p)| {
                                let x = if pts.len() == 1 { 50.0 } else { i as f64 / (pts.len() - 1) as f64 * 100.0 };
                                let y = 50.0 - ((p.pnl - mn) / (mx - mn) * 50.0);
                                format!("{x:.1},{y:.1}")
                            }).collect::<Vec<_>>().join(" ")
                        }
                    }
                />
            </svg>
        </div>
    }
}
