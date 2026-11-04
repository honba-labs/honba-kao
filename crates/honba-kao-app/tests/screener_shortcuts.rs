//! Keyboard shortcut contract tests (screener-honba-leptos.md §7):
//! `map_key_event` must map exactly the documented keys, stay inert under
//! ctrl/meta/alt chords and IME-free typing targets, and always dismiss on
//! Escape. Also pins the keyboard module's TS-side component surface.

use honba_kao_app::features::screener::components::ScreenCatalogModal;
use honba_kao_app::features::screener::keyboard::{ScreenerKeyAction, map_key_event};

#[test]
fn t1_primary_keys_map_to_actions() {
    assert_eq!(
        map_key_event(".", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::OpenCatalog)
    );
    assert_eq!(
        map_key_event("N", true, false, false, false, "BODY"),
        Some(ScreenerKeyAction::SaveScreen)
    );
    assert_eq!(
        map_key_event("n", true, false, false, false, "BODY"),
        Some(ScreenerKeyAction::SaveScreen)
    );
    assert_eq!(
        map_key_event("j", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::NextRow)
    );
    assert_eq!(
        map_key_event("k", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::PrevRow)
    );
    assert_eq!(
        map_key_event("ArrowDown", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::NextRow)
    );
    assert_eq!(
        map_key_event("ArrowUp", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::PrevRow)
    );
    assert_eq!(
        map_key_event(" ", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::ToggleStar)
    );
    assert_eq!(
        map_key_event("Enter", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::EnterRoute)
    );
    assert_eq!(
        map_key_event("f", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::OpenFilters)
    );
    assert_eq!(
        map_key_event("c", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::OpenColumns)
    );
    assert_eq!(
        map_key_event("h", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::ToggleView)
    );
    assert_eq!(
        map_key_event("Escape", false, false, false, false, "BODY"),
        Some(ScreenerKeyAction::Dismiss)
    );
}

#[test]
fn t2_modifier_chords_are_ignored() {
    assert!(
        map_key_event(".", false, true, false, false, "BODY").is_none(),
        "ctrl reserved for chrome hotkeys"
    );
    assert!(
        map_key_event("j", false, false, true, false, "BODY").is_none(),
        "meta reserved"
    );
    assert!(
        map_key_event("j", false, false, false, true, "BODY").is_none(),
        "alt reserved"
    );
    assert!(
        map_key_event("N", false, false, false, false, "BODY").is_none(),
        "unshifted N is not save"
    );
    assert!(map_key_event("x", false, false, false, false, "BODY").is_none());
}

#[test]
fn t3_typing_targets_swallow_everything_but_escape() {
    for tag in ["INPUT", "TEXTAREA", "SELECT", "input", "textarea"] {
        assert!(
            map_key_event("j", false, false, false, false, tag).is_none(),
            "typing context {tag}"
        );
        assert!(
            map_key_event("N", true, false, false, false, tag).is_none(),
            "typing context {tag}"
        );
        assert!(
            map_key_event(".", false, false, false, false, tag).is_none(),
            "typing context {tag}"
        );
    }
    assert_eq!(
        map_key_event("Escape", false, false, false, false, "INPUT"),
        Some(ScreenerKeyAction::Dismiss)
    );
    assert_eq!(
        map_key_event("j", false, false, false, false, "DIV"),
        Some(ScreenerKeyAction::NextRow)
    );
}

#[test]
fn t4_component_surface_available() {
    let _ = ScreenCatalogModal;
    let _ = ScreenerKeyAction::OpenCatalog;
    let _ = ScreenerKeyAction::Dismiss;
}
