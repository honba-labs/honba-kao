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

#[component]
pub fn HfRiskOMeter(
    level: RiskLevel,
) -> impl IntoView {
    let (angle_deg, level_label, color_class) = match level {
        RiskLevel::Low => (-75.0, "Low Risk", "text-emerald-400"),
        RiskLevel::LowToModerate => (-45.0, "Low to Moderate Risk", "text-teal-400"),
        RiskLevel::Moderate => (-15.0, "Moderate Risk", "text-cyan-400"),
        RiskLevel::ModeratelyHigh => (15.0, "Moderately High Risk", "text-amber-400"),
        RiskLevel::High => (45.0, "High Risk", "text-orange-400"),
        RiskLevel::VeryHigh => (75.0, "Very High Risk", "text-rose-500"),
    };

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
                        transform=format!("rotate({}, 50, 50)", angle_deg)
                        class=move || format!("transition-transform duration-300 {}", color_class)
                    />
                    <circle cx="50" cy="50" r="4" fill="currentColor" class=color_class />
                </svg>
            </div>
            <div class=format!("hf-riskometer-badge text-xs font-semibold {}", color_class)>
                {level_label}
            </div>
            <span class="text-[10px] text-text-muted">"SEBI Risk-o-meter Guidelines"</span>
        </div>
    }
}
