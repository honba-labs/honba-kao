//! `HfRiskOMeter` — SEBI-compliant 6-zone risk needle visualizer.
//! Ported from the restructure spec (Section 7.5).

use leptos::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    LowToModerate,
    Moderate,
    ModeratelyHigh,
    High,
    VeryHigh,
}

/// Pure mapping of risk level to (angle, label, css class). Extracted so it
/// can be unit-tested without a DOM.
pub(crate) fn risk_style(level: RiskLevel) -> (f64, String, String) {
    match level {
        RiskLevel::Low => (-75.0, "Low Risk".to_string(), "text-emerald-400".to_string()),
        RiskLevel::LowToModerate => (-45.0, "Low to Moderate Risk".to_string(), "text-teal-400".to_string()),
        RiskLevel::Moderate => (-15.0, "Moderate Risk".to_string(), "text-cyan-400".to_string()),
        RiskLevel::ModeratelyHigh => (15.0, "Moderately High Risk".to_string(), "text-amber-400".to_string()),
        RiskLevel::High => (45.0, "High Risk".to_string(), "text-orange-400".to_string()),
        RiskLevel::VeryHigh => (75.0, "Very High Risk".to_string(), "text-rose-500".to_string()),
    }
}

#[component]
pub fn HfRiskOMeter(
    #[prop(into)] level: Signal<RiskLevel>,
) -> impl IntoView {
    let style = Signal::derive(move || risk_style(level.get()));

    view! {
        <div class="hf-riskometer-widget flex flex-col items-center gap-1.5 p-3 rounded bg-surface-base border border-border-soft">
            <div class="hf-riskometer-dial relative w-32 h-16">
                <svg view_box="0 0 100 55" class="w-full h-full">
                    <path d="M 10 50 A 40 40 0 0 1 90 50" fill="none" stroke="var(--hf-bg-muted, #333)" stroke_width="8" />
                    <line
                        x1="50" y1="50" x2="50" y2="15"
                        stroke="currentColor"
                        stroke_width="2.5"
                        stroke_linecap="round"
                        transform=move || {
                            let (a, _, _) = style.get();
                            format!("rotate({}, 50, 50)", a)
                        }
                        class="transition-transform duration-300"
                    />
                    <circle
                        cx="50" cy="50" r="4" fill="currentColor"
                        class=move || {
                            let (_, _, c) = style.get();
                            c.clone()
                        }
                    />
                </svg>
            </div>
            <div class=move || {
                let (_, _, c) = style.get();
                format!("hf-riskometer-badge text-xs font-semibold {}", c)
            }>
                {move || {
                    let (_, l, _) = style.get();
                    l.clone()
                }}
            </div>
            <span class="text-[10px] text-text-muted">"SEBI Risk-o-meter Guidelines"</span>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_style_maps_each_level_to_angle_label_and_class() {
        let cases = [
            (RiskLevel::Low, -75.0, "Low Risk"),
            (
                RiskLevel::LowToModerate,
                -45.0,
                "Low to Moderate Risk",
            ),
            (RiskLevel::Moderate, -15.0, "Moderate Risk"),
            (
                RiskLevel::ModeratelyHigh,
                15.0,
                "Moderately High Risk",
            ),
            (RiskLevel::High, 45.0, "High Risk"),
            (RiskLevel::VeryHigh, 75.0, "Very High Risk"),
        ];
        for (level, angle, label) in cases {
            let (a, l, c) = risk_style(level);
            assert!((a - angle).abs() < f64::EPSILON);
            assert_eq!(l, label);
            assert!(!c.is_empty());
        }
        // Angles progress monotonically with risk severity.
        let angles = [
            risk_style(RiskLevel::Low).0,
            risk_style(RiskLevel::Moderate).0,
            risk_style(RiskLevel::High).0,
        ];
        assert!(angles[0] < angles[1] && angles[1] < angles[2]);
    }
}
