//! `HfCurrencyIcon`, `HfCurrencyListItem`, `HfAssetBadge` — currency / asset glyphs.
//! Ported from `secured-finance-app` atoms `CurrencyIcon`, `CurrencyItem`, `AssetInformation`.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Currency {
    Inr,
    Usd,
    Btc,
    Eth,
    Usdc,
}

impl Currency {
    fn symbol(self) -> &'static str {
        match self {
            Currency::Inr => "₹",
            Currency::Usd => "$",
            Currency::Btc => "₿",
            Currency::Eth => "Ξ",
            Currency::Usdc => "USDC",
        }
    }

    fn svg_path(self) -> &'static str {
        match self {
            Currency::Inr => "M21 6H3v12h15v-6h-6l3-6z",
            Currency::Usd => "M12 1a9 9 0 0 0-9 9 9 9 0 0 0 9 9 9 9 0 0 0 9-9 9 9 0 0 0-9-9Zm0 6h-4m4 4h4m-4-8V6m0 12V12m-4 6h4",
            Currency::Btc => "M14 8L12 10l2 2M20 12h-8l-2 2-2-2H4l2-2 2 2 2-2h8l2 2z",
            Currency::Eth => "M10 14l2-2 2 2M2 12h20M12 2v20",
            Currency::Usdc => "M12 2a9 9 0 0 0-9 9v2a9 9 0 0 0 9 9 9 9 0 0 0 9-9v-2a9 9 0 0 0-9-9Zm0 6h-4m4 4h4m-4-8v-6m0 12v-6",
        }
    }
}

/// SVG icon representing fiat or crypto currency (₹, $, BTC, ETH, USDC).
#[component]
pub fn HfCurrencyIcon(
    #[prop(into)] currency: Currency,
    #[prop(into, optional)] size: Option<&'static str>,
) -> impl IntoView {
    let size = size.unwrap_or("w-4 h-4");
    let path = currency.svg_path();
    let symbol = currency.symbol();
    view! {
        <span class=format!("hf-currency-icon {} text-text-high", size)>
            <svg class="hf-icon" view_box="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
                <path d=path />
            </svg>
            <span class="ml-1 text-[10px] font-mono">{symbol}</span>
        </span>
    }
}

/// Currency dropdown option showing icon, symbol, and available cash.
#[component]
pub fn HfCurrencyListItem(
    #[prop(into)] currency: Currency,
    #[prop(into)] available: String,
    #[prop(into, optional)] selected: Option<bool>,
) -> impl IntoView {
    let selected = selected.unwrap_or(false);
    view! {
        <div class=format!("hf-currency-list-item flex items-center gap-2 px-3 py-1.5 text-sm {}",
            if selected { "bg-emerald-600/20 text-text-high" } else { "text-text-high hover:bg-surface-subtle" }
        )>
            <HfCurrencyIcon currency />
            <span class="ml-auto text-text-muted">{available}</span>
        </div>
    }
}

/// Asset symbol, display name, and underlying contract/currency icon.
#[component]
pub fn HfAssetBadge(
    #[prop(into)] symbol: String,
    #[prop(into, optional)] name: Option<String>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let base = "hf-asset-badge inline-flex items-center gap-1.5 rounded border border-border-soft bg-surface-subtle px-2 py-0.5 text-xs font-mono font-bold text-text-high";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <span>{symbol}</span>
            {name.map(|n| view! { <span class="text-text-muted text-[10px]">{n}</span> })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_svg_paths_are_non_empty_and_well_formed() {
        for currency in [
            Currency::Inr,
            Currency::Usd,
            Currency::Btc,
            Currency::Eth,
            Currency::Usdc,
        ] {
            let path = currency.svg_path();
            assert!(!path.is_empty(), "{:?} svg path must not be empty", currency);
            // Every path must begin with a moveto and contain only valid command letters.
            let first = path.chars().next().unwrap_or(' ');
            assert_eq!(first, 'M', "{:?} svg path must start with absolute moveto", currency);
            for ch in path.chars() {
                assert!(
                    "MMLHCSVQTAZmmlhcsvqatz0123456789.-+ ".contains(ch),
                    "{:?} svg path contains invalid character {:?}",
                    currency,
                    ch
                );
            }
        }
    }
}
