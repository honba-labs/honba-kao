//! `HfRiskOMeter` — SEBI-compliant 6-zone risk level badge.
//! Ported from the restructure spec (Section 7.5). The needle dial is not
//! ported yet: nsh has no SVG or arc primitive, so only the zone's label and
//! colour are drawn (same fallback as `primitive/sparkline.rs`).

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum RiskLevel {
    #[default]
    Low,
    LowToModerate,
    Moderate,
    ModeratelyHigh,
    High,
    VeryHigh,
}

/// Pure mapping of risk level to (angle, label, css class). Extracted so it
/// can be unit-tested without a DOM.
pub fn risk_style(level: RiskLevel) -> (f64, String, String) {
    match level {
        RiskLevel::Low => (
            -75.0,
            "Low Risk".to_string(),
            "text-emerald-400".to_string(),
        ),
        RiskLevel::LowToModerate => (
            -45.0,
            "Low to Moderate Risk".to_string(),
            "text-teal-400".to_string(),
        ),
        RiskLevel::Moderate => (
            -15.0,
            "Moderate Risk".to_string(),
            "text-cyan-400".to_string(),
        ),
        RiskLevel::ModeratelyHigh => (
            15.0,
            "Moderately High Risk".to_string(),
            "text-amber-400".to_string(),
        ),
        RiskLevel::High => (45.0, "High Risk".to_string(), "text-orange-400".to_string()),
        RiskLevel::VeryHigh => (
            75.0,
            "Very High Risk".to_string(),
            "text-rose-500".to_string(),
        ),
    }
}

/// Props for [`HfRiskOMeter`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfRiskOMeterProps {
    pub level: Prop<RiskLevel>,
}

/// A risk-level badge: the zone's label, coloured by its class.
#[component]
pub fn HfRiskOMeter(props: HfRiskOMeterProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    // The needle angle from `risk_style` is unused until the dial is ported.
    let (_angle, label, color) = risk_style(props.level.get_untracked());

    view()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .p(px(12.0))
        .rounded(px(4.0))
        .surface_base()
        .border_soft()
        .child(
            view()
                .app_class(Some(SharedString::from(format!(
                    "hf-riskometer-badge text-xs font-semibold {color}"
                ))))
                .child(
                    text(label)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(12.0)),
                ),
        )
        .child(
            text("SEBI Risk-o-meter Guidelines")
                .text_muted()
                .text_size(px(10.0)),
        )
}
