//! Unit tests for `honba-kao-ui-core` primitives.
//!
//! These run on the host (no WASM/DOM required) and cover the deterministic,
//! pure-logic surface of each primitive: enum string/class/label mappings and
//! computed values. Interactive DOM rendering and event handling are covered
//! by the Playwright E2E suite in Phase 8.2.

use honba_kao_ui_core::*;

#[test]
fn hf_button_variant_as_str() {
    assert_eq!(HfButtonVariant::Default.as_str(), "default");
    assert_eq!(HfButtonVariant::Green.as_str(), "green");
    assert_eq!(HfButtonVariant::Red.as_str(), "red");
    assert_eq!(HfButtonVariant::Blue.as_str(), "blue");
    assert_eq!(HfButtonVariant::Ghost.as_str(), "ghost");
}

#[test]
fn hf_button_size_as_str() {
    assert_eq!(HfButtonSize::Small.as_str(), "sm");
    assert_eq!(HfButtonSize::Default.as_str(), "default");
    assert_eq!(HfButtonSize::Large.as_str(), "lg");
}

#[test]
fn hf_trader_state_class_and_label() {
    assert_eq!(HfTraderState::Running.as_class(), "running");
    assert_eq!(HfTraderState::Paused.as_class(), "paused");
    assert_eq!(HfTraderState::Stopped.as_class(), "stopped");

    assert_eq!(HfTraderState::Running.as_label(), "RUNNING");
    assert_eq!(HfTraderState::Paused.as_label(), "PAUSED");
    assert_eq!(HfTraderState::Stopped.as_label(), "STOPPED");
}

#[test]
fn hf_toast_variant_as_str() {
    assert_eq!(HfToastVariant::Success.as_str(), "success");
    assert_eq!(HfToastVariant::Error.as_str(), "error");
    assert_eq!(HfToastVariant::Info.as_str(), "info");
}

#[test]
fn hf_scrollbar_orientation_as_str() {
    assert_eq!(HfScrollbarOrientation::Vertical.as_str(), "vertical");
    assert_eq!(HfScrollbarOrientation::Horizontal.as_str(), "horizontal");
}

#[test]
fn hf_drawer_side_as_str() {
    assert_eq!(HfDrawerSide::Bottom.as_str(), "bottom");
    assert_eq!(HfDrawerSide::Right.as_str(), "right");
    assert_eq!(HfDrawerSide::Left.as_str(), "left");
    assert_eq!(HfDrawerSide::Top.as_str(), "top");
}

#[test]
fn hf_toolbar_variant_as_str() {
    assert_eq!(HfToolbarVariant::Default.as_str(), "default");
    assert_eq!(HfToolbarVariant::Condensed.as_str(), "condensed");
}

#[test]
fn hf_alert_variant_as_str() {
    assert_eq!(HfAlertVariant::Default.as_str(), "default");
    assert_eq!(HfAlertVariant::Info.as_str(), "info");
    assert_eq!(HfAlertVariant::Warning.as_str(), "warning");
    assert_eq!(HfAlertVariant::Destructive.as_str(), "destructive");
}

#[test]
fn hf_dialog_variant_as_str() {
    assert_eq!(HfDialogVariant::Default.as_str(), "default");
    assert_eq!(HfDialogVariant::Warning.as_str(), "warning");
    assert_eq!(HfDialogVariant::Confirm.as_str(), "confirm");
}

#[test]
fn hf_resizable_direction_as_str() {
    assert_eq!(HfResizableDirection::Horizontal.as_str(), "horizontal");
    assert_eq!(HfResizableDirection::Vertical.as_str(), "vertical");
}

#[test]
fn hf_segmented_control_variant_as_str() {
    assert_eq!(HfSegmentedControlVariant::Default.as_str(), "default");
    assert_eq!(HfSegmentedControlVariant::Inline.as_str(), "inline");
}
