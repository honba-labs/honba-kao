use honba_kao_ui_core::theme::tokens::class;

#[test]
fn class_constants_are_non_empty() {
    for class in [
        class::BG_CANVAS,
        class::BG_SURFACE,
        class::TEXT_HIGH,
        class::TEXT_MUTED,
        class::BORDER_SOFT,
        class::ROUNDED,
        class::TEXT_POSITIVE,
        class::TEXT_NEGATIVE,
    ] {
        assert!(!class.is_empty());
    }
}
