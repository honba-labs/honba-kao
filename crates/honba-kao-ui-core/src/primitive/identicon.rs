//! `HfIdenticon` — procedural account avatar glyph generated from wallet or broker address.
//! Ported from `secured-finance-app` atom `Identicon`.

use leptos::prelude::*;

/// Renders a deterministic identicon from an address string.
#[component]
pub fn HfIdenticon(
    #[prop(into)] address: String,
    #[prop(into, optional)] size: Option<&'static str>,
) -> impl IntoView {
    let size = size.unwrap_or("w-8 h-8");
    // Deterministic hue from address bytes.
    let hue = address
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_add(b as u32) * 31)
        % 360;
    let color_class = match hue {
        0..60 => "text-emerald-400",
        60..120 => "text-cyan-400",
        120..180 => "text-blue-400",
        180..240 => "text-purple-400",
        240..300 => "text-pink-400",
        _ => "text-amber-400",
    };
    let initial = address
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    view! {
        <div class=format!("hf-identicon {} rounded-full flex items-center justify-center text-xs font-bold bg-surface-elevated border border-border-soft {}", size, color_class)>
            {initial}
        </div>
    }
}
