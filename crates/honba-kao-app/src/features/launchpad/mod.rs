//! Launchpad (Phase 4.5) at `/equity/launchpad`, with a pop-out window per
//! panel at `/equity/launchpad/popout?panel=<slug>`. Mock data only.

mod launchpad_page;
mod popout_page;

pub use launchpad_page::LaunchpadPage;
pub use popout_page::PopoutPage;

/// The three placeholder slots as `(slug, title)`, shared by the mosaic and
/// the pop-out lookup so a slug means the same panel in both windows.
pub(crate) const PANELS: [(&str, &str); 3] = [
    ("chart", "Chart"),
    ("watchlist", "Watchlist"),
    ("news", "News"),
];

/// Title for a `panel` slug, `None` for an unknown or missing slug.
pub(crate) fn panel_title(slug: &str) -> Option<&'static str> {
    PANELS
        .iter()
        .find(|(known, _)| *known == slug)
        .map(|(_, title)| *title)
}
