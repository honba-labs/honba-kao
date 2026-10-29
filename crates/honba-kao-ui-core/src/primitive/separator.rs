//! `HfSeparator`, `HfColorBar`, `HfOiBar` — styled [`View`] primitives.

use nsh_core::prelude::*;

/// Orientation of a separator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Props for [`HfSeparator`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSeparatorProps {
    #[prop(default)]
    pub orientation: Prop<Option<SeparatorOrientation>>,
}

/// A thin divider line.
#[component]
pub fn HfSeparator(props: HfSeparatorProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let orient = props.orientation.get_untracked().unwrap_or_default();

    if matches!(orient, SeparatorOrientation::Vertical) {
        view().w(px(1.0)).flex_1().bg(rgb(0x232a3a))
    } else {
        view().h(px(1.0)).flex_1().bg(rgb(0x232a3a))
    }
}

/// One segment of a color bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, JsObject)]
pub struct ColorBarSegment {
    /// `0xRRGGBB`, the same color in both appearances (`AdaptiveColor`
    /// itself isn't `JsValue`, so it can't cross the props boundary).
    pub color: u32,
    pub pct: f32,
}

/// Props for [`HfColorBar`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfColorBarProps {
    pub segments: Prop<Vec<ColorBarSegment>>,
    #[prop(default = false)]
    pub vertical: Prop<bool>,
}

/// A colored bar split into weighted segments (e.g. for liquidation /
/// exposure heat).
#[component]
pub fn HfColorBar(props: HfColorBarProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let vertical = props.vertical.get_untracked();
    let segs = props.segments.get_untracked();

    let base = if vertical {
        view().flex().flex_col()
    } else {
        view().flex()
    };

    base.children(segs.into_iter().map(|seg| {
        view()
            .flex_grow(seg.pct.max(0.0))
            .h(px(8.0))
            .w(px(8.0))
            .bg(AdaptiveColor::rgb(seg.color, seg.color))
    }))
}

/// Props for [`HfOiBar`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfOiBarProps {
    /// Call-side open interest, as a fraction (0.0–1.0) of the total.
    pub call_pct: Prop<f32>,
}

/// An open-interest skew bar: call share (positive) vs. put share
/// (negative).
#[component]
pub fn HfOiBar(props: HfOiBarProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let call = props.call_pct.get_untracked().clamp(0.0, 1.0);

    view()
        .flex()
        .h(px(4.0))
        .rounded(px(2.0))
        .child(view().flex_grow(call).bg(rgb(0x10b981)))
        .child(view().flex_grow(1.0 - call).bg(rgb(0xef4444)))
}
