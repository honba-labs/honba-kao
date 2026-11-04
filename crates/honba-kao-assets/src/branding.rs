//! Branding & graphic media: high-DPI logos, the native window icon,
//! the web favicon, and the exchange / broker emblem badges.
//!
//! Every file is an `asset!` constant so the NSH registry ships it and
//! the engine resolves it before the first render. Images carry a
//! [`Size<Points>`]-based intrinsic size; the `@2x` logos declare their
//! 2-pixel-per-point density via the filename (no density-suffixed
//! siblings are staged next to them — keep it that way).

use nsh_core::element::{FileAsset, ImageAsset, asset};

/// The 2x high-DPI platform logo (`logo@2x.png`: 2 pixels per point),
/// used by splash/about panes and canvas demos.
pub const LOGO_2X: ImageAsset = asset!("branding/logo@2x.png");

/// The greyscale / stencil logo for dark title bars and etched headers.
pub const LOGO_MONO_2X: ImageAsset = asset!("branding/logo-monochrome@2x.png");

/// The 512x512 native desktop window icon.
pub const APP_ICON: ImageAsset = asset!("branding/app-icon.png");

/// The web-target favicon (multi-size `.ico`). NSH knows images as
/// `.png`/`.jpg`/`.jpeg`/`.gif`/`.webp`/`.heic`/`.svg` — an `.ico` is a
/// [`FileAsset`] (shipped by [`crate::registry`], resolvable by its
/// `asset:` URI); web shells bind it via the HTML `<link rel="icon">` layer.
pub const FAVICON: FileAsset = asset!("branding/favicon.ico");

pub mod exchanges {
    use super::*;

    /// National Stock Exchange of India badge.
    pub const NSE: ImageAsset = asset!("emblems/exchanges/nse.png");

    /// BSE (formerly Bombay Stock Exchange) badge.
    pub const BSE: ImageAsset = asset!("emblems/exchanges/bse.png");

    /// MCX (Multi Commodity Exchange) badge.
    pub const MCX: ImageAsset = asset!("emblems/exchanges/mcx.png");
}

pub mod brokers {
    use super::*;

    /// Zerodha badge.
    pub const ZERODHA: ImageAsset = asset!("emblems/brokers/zerodha.png");

    /// Interactive Brokers badge.
    pub const INTERACTIVE_BROKERS: ImageAsset = asset!("emblems/brokers/interactive_brokers.png");

    /// OpenAlgo badge.
    pub const OPENALGO: ImageAsset = asset!("emblems/brokers/openalgo.png");
}
