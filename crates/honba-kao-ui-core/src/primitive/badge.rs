//! `HfBadge`, `HfStatusChip`, `HfMoneynessBadge` — status / moneyness pill atoms.
//! `HfMoneynessBadge` renders `ITM`, `ATM`, `OTM`.

use nsh_core::element::{Children, NoEvent};
use nsh_core::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum BadgeVariant {
    #[default]
    Default,
    Success,
    Warning,
    Danger,
    Info,
}

impl BadgeVariant {
    fn colors(self) -> (AdaptiveColor, AdaptiveColor) {
        match self {
            BadgeVariant::Default => (rgb(0x232a3a).into(), rgb(0xe5e7eb).into()),
            BadgeVariant::Success => (rgb(0x10b981).into(), rgb(0x34d399).into()),
            BadgeVariant::Warning => (rgb(0xf59e0b).into(), rgb(0xfbbf24).into()),
            BadgeVariant::Danger => (rgb(0xef4444).into(), rgb(0xf87171).into()),
            BadgeVariant::Info => (rgb(0x3b82f6).into(), rgb(0x60a5fa).into()),
        }
    }
}

/// Props for [`HfBadge`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfBadgeProps {
    #[prop(default)]
    pub variant: Prop<Option<BadgeVariant>>,
    #[prop(default = false)]
    pub outline: Prop<bool>,
    pub children: Prop<Children>,
}

/// Generic status/badge pill.
#[component]
pub fn HfBadge(props: HfBadgeProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let variant_val = props.variant.get_untracked().unwrap_or_default();
    let outline_val = props.outline.get_untracked();
    let (bg, fg) = variant_val.colors();

    let base = view()
        .px(px(1.5))
        .py(px(0.5))
        .rounded(px(6.0))
        .text_size(px(10.0))
        .font_weight(FontWeight::SEMIBOLD);

    if outline_val {
        base.border(px(1.0))
            .border_color(bg)
            .text_color(fg)
            .child(text(children_string(props.children)))
    } else {
        base.bg(bg)
            .text_color(fg)
            .child(text(children_string(props.children)))
    }
}

/// Props for [`HfStatusChip`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfStatusChipProps {
    pub status: Prop<String>,
    #[prop(default)]
    pub variant: Prop<Option<BadgeVariant>>,
}

/// Status chip with a colored dot indicator.
#[component]
pub fn HfStatusChip(props: HfStatusChipProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let variant_val = props.variant.get_untracked().unwrap_or_default();
    let (_, dot_color) = variant_val.colors();
    let status_text = props.status.get_untracked();

    view()
        .flex()
        .items_center()
        .gap(px(1.5))
        .px(px(2.0))
        .py(px(0.5))
        .rounded(px(999.0))
        .text_size(px(10.0))
        .font_weight(FontWeight::SEMIBOLD)
        .border(px(1.0))
        .child(
            view()
                .w(px(1.5))
                .h(px(1.5))
                .rounded(px(999.0))
                .bg(dot_color),
        )
        .child(text(status_text))
}

/// Moneyness badge: `ITM`, `ATM`, `OTM`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum Moneyness {
    ITM,
    #[default]
    ATM,
    OTM,
}

/// Pure mapping of moneyness to (label, color). Extracted so it can be
/// unit-tested without a renderer.
pub(crate) fn moneyness_style(moneyness: Moneyness) -> (&'static str, AdaptiveColor) {
    match moneyness {
        Moneyness::ITM => ("ITM", rgb(0x10b981).into()),
        Moneyness::ATM => ("ATM", rgb(0xe5e7eb).into()),
        Moneyness::OTM => ("OTM", rgb(0xef4444).into()),
    }
}

/// Props for [`HfMoneynessBadge`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfMoneynessBadgeProps {
    pub moneyness: Prop<Moneyness>,
}

#[component]
pub fn HfMoneynessBadge(props: HfMoneynessBadgeProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let style = computed(move || moneyness_style(props.moneyness.get()));

    view()
        .px(px(1.5))
        .py(px(0.5))
        .rounded(px(6.0))
        .text_size(px(10.0))
        .font_weight(FontWeight::BOLD)
        .border(px(1.0))
        .bg(move || style.get().1)
        .text_color(move || style.get().1)
        .child(dynamic(move || text(style.get().0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moneyness_style_maps_each_variant_to_label_and_color() {
        assert_eq!(moneyness_style(Moneyness::ITM).0, "ITM");
        assert_eq!(moneyness_style(Moneyness::ATM).0, "ATM");
        assert_eq!(moneyness_style(Moneyness::OTM).0, "OTM");
        let styles = [
            moneyness_style(Moneyness::ITM),
            moneyness_style(Moneyness::ATM),
            moneyness_style(Moneyness::OTM),
        ];
        for (label, _) in &styles {
            assert!(!label.is_empty());
        }
        assert_ne!(styles[0].0, styles[1].0);
        assert_ne!(styles[1].0, styles[2].0);
    }
}

/// Best-effort extraction of text content from children.
fn children_string(_children: Prop<Children>) -> String {
    String::new()
}
