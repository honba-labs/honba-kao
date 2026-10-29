//! `HfButton`, `HfIconButton`, `HfCloseButton`, `HfChip` — wrapping
//! [`nsh_base::Button`].
//!
//! `nsh_base::Button` owns press/disabled/selected behavior and leaves the
//! look to the app. Honba supplies colors, padding and radius through the
//! [`HonbaStyle`] trait (`crate::theme::tokens::HonbaStyle`).
//!
//! `nsh_base::Button`'s own event (`ButtonEvent::Press` / `PressDown` /
//! `PressUp`) is mirrored locally as [`HfButtonEvent`] and used as every
//! component's event here instead of a `Fn()` callback prop: a
//! `#[derive(Props)]` struct must be `Clone + PartialEq`
//! (`nsh_core::element::Props: Clone + PartialEq`), which a boxed closure
//! can never satisfy — and `#[derive(Event)]`'s own hidden macro of the
//! enum's name (which `#[component]` expands) means `nsh_base::ButtonEvent`
//! can never be reused directly as a `Scope<T>` declared in this crate
//! either, so [`HfButtonEvent`] translates it 1:1. Composing code hears
//! presses the same way it would on a raw `nsh_base::Button`:
//! `.on_press(move || ..)` on the element this file's constructors make.

use nsh_base::{Button, ButtonEvent, ButtonProps};
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// The events of every button-like component in this file (`HfButton`,
/// `HfIconButton`, `HfCloseButton`, `HfChip`), translated 1:1 from
/// [`nsh_base::ButtonEvent`]: `#[derive(Event)]` makes a hidden macro of
/// the enum's name which the component's `#[component]` expands, so the
/// event enum and the component using it as `Scope<T>` must be of one
/// crate — `nsh_base::ButtonEvent` can never be reused directly here.
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfButtonEvent {
    /// The button was pressed.
    Press,
    /// A press went down on it (for the app's own press feedback).
    PressDown,
    /// The press ended: released, moved away or cancelled.
    PressUp,
}

impl From<ButtonEvent> for HfButtonEvent {
    fn from(event: ButtonEvent) -> Self {
        match event {
            ButtonEvent::Press => HfButtonEvent::Press,
            ButtonEvent::PressDown => HfButtonEvent::PressDown,
            ButtonEvent::PressUp => HfButtonEvent::PressUp,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Ghost,
    Danger,
    Outline,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum ButtonSize {
    Xxs,
    Xs,
    #[default]
    Sm,
    Md,
    Lg,
}

/// The background / text colors of a button variant, ported from
/// `secured-finance-app`'s `border-transparent bg-emerald-600 text-white`
/// (Primary), `bg-transparent text-emerald-400` (Secondary), etc. Reuses
/// [`HonbaStyle`]'s semantic text tones where one already matches.
fn styled_for_variant<S: Styled>(el: S, variant: ButtonVariant) -> S {
    match variant {
        ButtonVariant::Primary => el
            .bg(AdaptiveColor::rgb(0x10b981, 0x10b981))
            .text_color(AdaptiveColor::rgb(0xffffff, 0xffffff)),
        ButtonVariant::Secondary => el.text_positive(),
        ButtonVariant::Tertiary => el.text_high(),
        ButtonVariant::Ghost => el.text_muted(),
        ButtonVariant::Danger => el
            .bg(AdaptiveColor::rgb(0xef4444, 0xef4444))
            .text_color(AdaptiveColor::rgb(0xffffff, 0xffffff)),
        ButtonVariant::Outline => el.text_high(),
    }
}

/// Padding and radius of a button size (`xxs … lg`, as the original
/// Leptos `ButtonSize::class`).
fn styled_for_size<S: Styled>(el: S, size: ButtonSize) -> S {
    match size {
        ButtonSize::Xxs => el.px(px(2.0)).py(px(0.5)).rounded(px(6.0)),
        ButtonSize::Xs => el.px(px(2.5)).py(px(1.0)).rounded(px(6.0)),
        ButtonSize::Sm => el.px(px(3.0)).py(px(1.0)).rounded_md(),
        ButtonSize::Md => el.px(px(3.5)).py(px(2.5)).rounded_md(),
        ButtonSize::Lg => el.px(px(5.0)).py(px(3.0)).rounded(px(8.0)),
    }
}

/// Padding and radius of an icon button's square hit target.
fn icon_styled_for_size<S: Styled>(el: S, size: ButtonSize) -> S {
    match size {
        ButtonSize::Xxs => el.p(px(0.5)).rounded_md(),
        ButtonSize::Xs => el.p(px(1.0)).rounded_md(),
        ButtonSize::Sm => el.p(px(1.5)).rounded_md(),
        ButtonSize::Md => el.p(px(2.0)).rounded_md(),
        ButtonSize::Lg => el.p(px(3.0)).rounded(px(12.0)),
    }
}

/// Props for [`HfButton`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfButtonProps {
    /// Honored by callers: the base widget is always a pressable button,
    /// never an `<a>`, so a caller wanting a link renders one itself and
    /// uses this only to decide whether to.
    #[prop(default)]
    pub href: Prop<Option<String>>,
    #[prop(default)]
    pub size: Prop<Option<ButtonSize>>,
    #[prop(default)]
    pub variant: Prop<Option<ButtonVariant>>,
    #[prop(default)]
    pub full_width: Prop<bool>,
    #[prop(default)]
    pub disabled: Prop<bool>,
    #[prop(default)]
    pub aria_label: Prop<Option<String>>,
    /// Extra Tailwind classes, applied after the button's own
    /// (`ComponentElement::app_class`).
    #[prop(default)]
    pub class: Prop<Option<String>>,
    pub children: Prop<Children>,
}

/// Primary action button.
#[component]
pub fn HfButton(props: HfButtonProps, cx: Scope<HfButtonEvent>) -> impl IntoElement {
    let size_val = props.size.get_untracked().unwrap_or_default();
    let variant_val = props.variant.get_untracked().unwrap_or_default();
    let full_width = props.full_width.get_untracked();
    let label = props
        .aria_label
        .get_untracked()
        .unwrap_or_else(|| "Button".to_string());
    let class = props.class;

    let el = Button::element(ButtonProps {
        disabled: props.disabled,
        accessibility_label: Prop::from(Some(SharedString::from(label))),
        role: Prop::from(Some(AccessibilityRole::Button)),
        children: props.children,
        ..Default::default()
    })
    .child(props.children)
    .on_event(move |event: &ButtonEvent| cx.emit(HfButtonEvent::from(event.clone())))
    .disabled_style(|s| s.opacity(0.5))
    .app_class(move || class.get().map(SharedString::from));

    let el = styled_for_variant(el, variant_val);
    let el = styled_for_size(el, size_val);
    if full_width { el.w_full() } else { el }
}

/// Props for [`HfIconButton`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfIconButtonProps {
    #[prop(default)]
    pub size: Prop<Option<ButtonSize>>,
    #[prop(default)]
    pub variant: Prop<Option<ButtonVariant>>,
    #[prop(default)]
    pub disabled: Prop<bool>,
    pub aria_label: Prop<String>,
    pub children: Prop<Children>,
}

/// Icon button with a standardized square hit target.
#[component]
pub fn HfIconButton(props: HfIconButtonProps, cx: Scope<HfButtonEvent>) -> impl IntoElement {
    let size_val = props.size.get_untracked().unwrap_or_default();
    let variant_val = props
        .variant
        .get_untracked()
        .unwrap_or(ButtonVariant::Ghost);

    let el = Button::element(ButtonProps {
        disabled: props.disabled,
        accessibility_label: Prop::from(Some(SharedString::from(props.aria_label.get_untracked()))),
        role: Prop::from(Some(AccessibilityRole::Button)),
        children: props.children,
        ..Default::default()
    })
    .child(props.children)
    .on_event(move |event: &ButtonEvent| cx.emit(HfButtonEvent::from(event.clone())))
    .disabled_style(|s| s.opacity(0.5));

    let el = styled_for_variant(el, variant_val);
    icon_styled_for_size(el, size_val)
}

/// Props for [`HfCloseButton`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCloseButtonProps {
    #[prop(default)]
    pub aria_label: Prop<Option<String>>,
}

/// Standardized close icon button (×): a ghost, small [`HfIconButton`].
#[component]
pub fn HfCloseButton(props: HfCloseButtonProps, cx: Scope<HfButtonEvent>) -> impl IntoElement {
    let label = props
        .aria_label
        .get_untracked()
        .unwrap_or_else(|| "Close".to_string());

    HfIconButton::element(HfIconButtonProps {
        size: Prop::from(Some(ButtonSize::Sm)),
        variant: Prop::from(Some(ButtonVariant::Ghost)),
        aria_label: Prop::from(label),
        ..Default::default()
    })
    .child(text("✕"))
    .on_event(move |event: &HfButtonEvent| cx.emit(event.clone()))
}

/// Props for [`HfChip`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfChipProps {
    #[prop(default = false)]
    pub active: Prop<bool>,
    pub children: Prop<Children>,
}

/// Pill-shaped tag button (e.g. for timeframes, filters, quick tags).
#[component]
pub fn HfChip(props: HfChipProps, cx: Scope<HfButtonEvent>) -> impl IntoElement {
    // `active` was a plain `bool` (not a signal) in the original Leptos
    // component too: read once, same as there.
    let active_val = props.active.get_untracked();
    let el = view().rounded(px(999.0));
    let el = if active_val {
        el.bg(AdaptiveColor::rgb(0x10b981, 0x10b981).opacity(0.1))
            .text_positive()
    } else {
        el.surface_muted().text_muted()
    };
    el.child(props.children)
        .on_press(move |_: &()| cx.emit(HfButtonEvent::Press))
}
