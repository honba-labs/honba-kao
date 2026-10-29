//! Page layout template — NSH port per nsh-port.md §4 Phase 4.

use nsh_core::prelude::*;

/// Props for [`HfPageAlertSlot`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPageAlertSlotProps {
    pub children: Prop<Children>,
}

/// Generic alert-banner slot at the top of an institutional page. For an
/// actual managed alert stack (severity-tagged entries, dismiss events),
/// use [`crate::molecules::core::HfAlertBannerStack`] as the slot's child
/// instead of adding stack semantics here.
#[component]
pub fn HfPageAlertSlot(props: HfPageAlertSlotProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(2.0))
        .w_full()
        .child(props.children)
}

/// `HfPage` layout template — standard container enforcing margins,
/// responsive max-widths, optional alert banner slot, and standardized page header.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPageProps {
    #[prop(default)]
    pub title: Prop<Option<String>>,
    #[prop(default)]
    pub subtitle: Prop<Option<String>>,
    #[prop(default)]
    pub max_width_class: Prop<String>,
    pub children: Prop<Children>,
}

/// Standard page container.
#[component]
pub fn HfPage(props: HfPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(4.0))
        .px_4()
        .py_3()
        .w_full()
        .child(
            view()
                .flex_row()
                .gap(px(2.0))
                .child(text(move || {
                    props.title.get_untracked().unwrap_or_default()
                }))
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD),
        )
        .child(props.children)
}
