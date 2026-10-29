use honba_kao_ui_core::molecules::wealth::risk_o_meter::{RiskLevel, risk_style};

#[test]
fn risk_style_maps_each_level_to_angle_label_and_class() {
    let cases = [
        (RiskLevel::Low, -75.0, "Low Risk"),
        (RiskLevel::LowToModerate, -45.0, "Low to Moderate Risk"),
        (RiskLevel::Moderate, -15.0, "Moderate Risk"),
        (RiskLevel::ModeratelyHigh, 15.0, "Moderately High Risk"),
        (RiskLevel::High, 45.0, "High Risk"),
        (RiskLevel::VeryHigh, 75.0, "Very High Risk"),
    ];
    for (level, angle, label) in cases {
        let (a, l, c) = risk_style(level);
        assert!((a - angle).abs() < f64::EPSILON);
        assert_eq!(l, label);
        assert!(!c.is_empty());
    }
    // Angles progress monotonically with risk severity.
    let angles = [
        risk_style(RiskLevel::Low).0,
        risk_style(RiskLevel::Moderate).0,
        risk_style(RiskLevel::High).0,
    ];
    assert!(angles[0] < angles[1] && angles[1] < angles[2]);
}
