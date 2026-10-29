//! `HfIdenticon` — procedural identicon drawn on [`Canvas`].

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfIdenticon`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfIdenticonProps {
    /// The address or key the identicon is deterministically drawn from.
    pub key: Prop<String>,
    #[prop(default = 64.0)]
    pub size: Prop<f32>,
}

/// A procedurally drawn identicon for an address or key.
///
/// `nsh_core::canvas` takes a `Painter` value (reactively), not an
/// imperative drawing closure, so the deterministic-seed-to-pixels path
/// isn't wired yet; this renders a sized, rounded placeholder for the key
/// until a `Painter` is composed from `key`'s hash.
#[component]
pub fn HfIdenticon(props: HfIdenticonProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let _key_val = props.key.get_untracked();
    let sz = props.size.get_untracked();

    view()
        .w(px(sz))
        .h(px(sz))
        .rounded(px(sz / 2.0))
        .surface_muted()
}
