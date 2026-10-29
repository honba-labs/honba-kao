//! `HfCurrency`, `HfAssetBadge`, `HfCurrencyIcon`, `HfCurrencyListItem` —
//! [`Text`] + formatting primitives.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Fiat or crypto currency a badge, icon, or list item is denominated in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum Currency {
    #[default]
    Inr,
    Usd,
    Btc,
    Eth,
    Usdc,
}

impl Currency {
    /// The glyph shown in place of a symbol override (₹, $, USDC, ...).
    pub fn symbol(self) -> &'static str {
        match self {
            Currency::Inr => "₹",
            Currency::Usd => "$",
            Currency::Btc => "₿",
            Currency::Eth => "Ξ",
            Currency::Usdc => "USDC",
        }
    }
}

/// Props for [`HfAssetBadge`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfAssetBadgeProps {
    pub currency: Prop<Currency>,
    #[prop(default)]
    pub symbol: Prop<Option<String>>,
}

/// A currency badge with an adornment (₹, $, USDC).
#[component]
pub fn HfAssetBadge(props: HfAssetBadgeProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let sym = props.symbol.get_untracked().unwrap_or_default();
    let cur = props.currency.get_untracked();

    view()
        .flex()
        .items_center()
        .gap(px(2.0))
        .px(px(1.5))
        .py(px(0.5))
        .rounded(px(6.0))
        .surface_muted()
        .child(
            text(if sym.is_empty() {
                cur.symbol().to_string()
            } else {
                sym
            })
            .text_high(),
        )
}

/// Props for [`HfCurrencyIcon`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCurrencyIconProps {
    /// The glyph shown in the circle.
    pub symbol: Prop<String>,
}

/// A currency icon (single letter in a circle).
#[component]
pub fn HfCurrencyIcon(props: HfCurrencyIconProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let sym = props.symbol.get_untracked();

    view()
        .w(px(6.0))
        .h(px(6.0))
        .rounded_full()
        .surface_elevated()
        .flex()
        .items_center()
        .justify_center()
        .child(text(sym).text_size(px(8.0)).text_positive())
}

/// Props for [`HfCurrencyListItem`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCurrencyListItemProps {
    pub symbol: Prop<String>,
    /// The amount, pre-formatted by the caller (who holds the
    /// `rust_decimal::Decimal` and applies the Zero Floats Policy; this
    /// leaf display primitive only renders the resulting text).
    pub amount: Prop<String>,
}

/// A currency list item (symbol + amount).
#[component]
pub fn HfCurrencyListItem(props: HfCurrencyListItemProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let sym = props.symbol.get_untracked();
    let amt = props.amount.get_untracked();

    view()
        .flex()
        .justify_between()
        .px(px(2.0))
        .py(px(1.0))
        .child(text(sym).text_high())
        .child(text(amt).text_muted().font_family("monospace"))
}
