//! `HfSortArrows`, `SortDirection` — sort indicator primitive.

use nsh_core::prelude::*;

/// Sort direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum SortDirection {
    #[default]
    None,
    Asc,
    Desc,
}

/// Props for [`HfSortArrows`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSortArrowsProps {
    pub direction: Prop<SortDirection>,
}

/// Sort arrows indicator (▲ / ▼).
#[component]
pub fn HfSortArrows(props: HfSortArrowsProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let dir = props.direction.get_untracked();

    let (arrow, color) = match dir {
        SortDirection::Asc => ("▲", rgb(0x10b981)),
        SortDirection::Desc => ("▼", rgb(0xef4444)),
        SortDirection::None => ("", rgb(0x64748b)),
    };

    view()
        .text_size(px(10.0))
        .text_color(color)
        .child(text(arrow))
}
