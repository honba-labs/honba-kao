//! `HfPlot`, `HfLine`, `HfGrid` — wrapping [`nsh_base::plot`]'s `Plot`,
//! `Line` and `Grid`. `Area` is not wrapped yet (no molecule needs it).
//!
//! `nsh_base::plot` draws its marks with no look of its own: a mark paints
//! with the CSS paint properties of the view it sits in, inherited from the
//! plot as CSS inherits them. So a line's stroke and a grid's dashes are set
//! here on the `HfLine` / `HfGrid` element itself, through the same
//! [`Styled`] builder every other shim uses, and the caller's `class` is
//! applied after them (`ComponentElement::app_class`).
//!
//! `HfPlot` does not surface [`nsh_base::plot::PlotEvent::HoverChange`]: no
//! molecule needs the hovered row yet, so the plot takes no event type
//! (`NoEvent`). Add an `HfPlotEvent` mirror when one does.
//!
//! Only the props the first chart molecule needs are exposed. More (motion,
//! tooltip, cross line, accessibility) can be added as later molecules need
//! them.

pub use nsh_base::plot::Curve;

use nsh_base::plot::{Grid, GridProps, Line, LineProps, Plot, PlotProps};
use nsh_core::element::Style;
use nsh_core::element::{Children, NoEvent, RowAccessor, Rows};
use nsh_core::prelude::*;
use nsh_core::{PatchValue, StyleDecl, StyleKey};

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfPlot`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPlotProps {
    /// The rows of data, one object per point (`nsh_base`'s `data`).
    pub data: Prop<Rows>,
    /// What places a row along the x axis (a field name). Without one, the
    /// row's index.
    #[prop(default)]
    pub x: Prop<Option<RowAccessor>>,
    /// What the marks draw along the y axis unless they name their own (a
    /// field name). Without one, the marks' own values are used.
    #[prop(default)]
    pub y: Prop<Option<RowAccessor>>,
    /// The marks (`HfLine`, `HfGrid`), added with `.child(..)`.
    pub children: Prop<Children>,
}

/// A chart plot: the rows, the axes and the marks drawn over them. It fills
/// its parent, so the caller sizes the parent.
#[component]
pub fn HfPlot(props: HfPlotProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    Plot::element(PlotProps {
        data: props.data,
        x: props.x,
        y: props.y,
        children: props.children,
        ..Default::default()
    })
    .child(props.children)
    .w_full()
    .h_full()
}

/// Props for [`HfLine`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfLineProps {
    /// How the line connects its points (`Curve::Natural` by default, a
    /// smooth curve; `Curve::Linear` for straight segments).
    #[prop(default)]
    pub curve: Prop<Curve>,
    /// Extra Tailwind classes, applied after the line's own styles
    /// (`ComponentElement::app_class`), e.g. to override its stroke.
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// A series drawn as a line through the plot's points, in emerald and 1.5
/// points wide unless `class` says otherwise.
#[component]
pub fn HfLine(props: HfLineProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let class = props.class;
    Line::element(LineProps {
        curve: props.curve,
        ..Default::default()
    })
    .stroke_color(AdaptiveColor::rgb(0x10b981, 0x10b981))
    .stroke_width(px(1.5))
    .app_class(move || class.get().map(SharedString::from))
}

/// Props for [`HfGrid`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfGridProps {
    /// Vertical lines, at these values of the plot's x scale.
    #[prop(default)]
    pub x: Prop<Vec<f64>>,
    /// Horizontal lines, at these values of the plot's y scale. `vec![0.0]`
    /// draws a line at y = 0 (e.g. a breakeven axis).
    #[prop(default)]
    pub y: Prop<Vec<f64>>,
    /// Extra Tailwind classes, applied after the grid's own styles
    /// (`ComponentElement::app_class`).
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// Lines across the plot: 1 point wide, dashed, in the warning tone (the
/// stroke is `currentColor`, which the warning text color sets) at 70%
/// opacity, unless `class` says otherwise.
#[component]
pub fn HfGrid(props: HfGridProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let class = props.class;
    // `strokeDasharray` has no typed `Styled` setter, so it goes in as a
    // declaration, applied with `inline_style`.
    let dashed = Style::from_decls([StyleDecl::Set(
        StyleKey::StrokeDasharray,
        PatchValue::Str("4 3".into()),
    )]);
    Grid::element(GridProps {
        x: props.x,
        y: props.y,
        ..Default::default()
    })
    .text_warning()
    .opacity(0.7)
    .inline_style(dashed)
    .app_class(move || class.get().map(SharedString::from))
}
