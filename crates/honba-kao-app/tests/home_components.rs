//! Unit tests for the Mission Control composite panels (plan-1.md Milestone 2).
//!
//! Covers the deterministic, pure-logic surfaces of the `features/home`
//! components: AiInsightCard, IdeasRadar, PortfolioMiniChart, IntelWire,
//! ResultsSummaryCards and IntelligenceTimeline.

use honba_kao_app::features::home::components::*;

#[test]
fn insight_engine_label_and_tones() {
    // Live engines name the actual model when known, else the engine id.
    assert_eq!(engine_label("lm-studio", Some("gemma-3-4b")), "gemma-3-4b");
    assert_eq!(engine_label("lm-studio", None), "lm-studio");
    assert_eq!(engine_label("lm-studio", Some("  ")), "lm-studio");

    // Unavailable / empty engines degrade to the static label.
    assert_eq!(engine_label("unavailable", Some("gemma")), "AI unavailable");
    assert_eq!(engine_label("", None), "AI unavailable");

    assert_eq!(InsightTone::Positive.as_str(), "positive");
    assert_eq!(InsightTone::Neutral.as_str(), "neutral");
    assert_eq!(InsightTone::Negative.as_str(), "negative");

    assert_eq!(AiInsightStatus::Idle.as_str(), "idle");
    assert_eq!(AiInsightStatus::Loading.as_str(), "loading");
    assert_eq!(AiInsightStatus::Done.as_str(), "done");
    assert_eq!(AiInsightStatus::Error.as_str(), "error");

    let data = InsightData::new("lm-studio")
        .with_model("gemma-3-4b")
        .with_summary("Risk-on regime with broadening participation.")
        .with_sections(vec![InsightSection::new(
            "Regime classification",
            InsightTone::Positive,
            vec!["Nifty above 50-DMA; breadth improving.".to_string()],
        )]);
    assert_eq!(data.engine, "lm-studio");
    assert_eq!(data.model.as_deref(), Some("gemma-3-4b"));
    assert_eq!(data.sections.len(), 1);
    assert_eq!(data.sections[0].points.len(), 1);
}

#[test]
fn ideas_radar_grouping_limits_and_trigger_tones() {
    assert_eq!(
        IdeaSignalCategory::VolumeShockers.label(),
        "Volume Shockers"
    );
    assert_eq!(IdeaSignalCategory::Breakout52W.label(), "52W Breakouts");
    assert_eq!(IdeaSignalCategory::MeanReversion.label(), "Mean Reversion");
    assert_eq!(
        IdeaSignalCategory::from_label("52W Breakouts"),
        Some(IdeaSignalCategory::Breakout52W)
    );
    assert_eq!(IdeaSignalCategory::from_label("nonsense"), None);

    // Trigger tone follows the leading sign.
    assert_eq!(trigger_tone(Some("+3.8x vol")), "up");
    assert_eq!(trigger_tone(Some("-1.2% drift")), "down");
    assert_eq!(trigger_tone(Some("flat")), "neutral");
    assert_eq!(trigger_tone(None), "neutral");

    let mk = |id: &str, category: IdeaSignalCategory, symbol: &str| {
        IdeaSignal::new(id, category, "Signal headline").with_symbol(symbol)
    };
    let signals = vec![
        mk("a", IdeaSignalCategory::VolumeShockers, "RELIANCE"),
        mk("b", IdeaSignalCategory::VolumeShockers, "TCS"),
        mk("c", IdeaSignalCategory::VolumeShockers, "INFY"), // dropped: 3rd in category
        mk("d", IdeaSignalCategory::Breakout52W, "HDFCBANK"),
        mk("e", IdeaSignalCategory::MeanReversion, "ICICIBANK"),
        mk("f", IdeaSignalCategory::MeanReversion, "SBIN"),
        mk("g", IdeaSignalCategory::MeanReversion, "AXISBANK"), // dropped: category capped
    ];

    let entries = build_radar_entries(signals, 4, 2);
    assert_eq!(entries.len(), 5);
    assert_eq!(entries[0].category, IdeaSignalCategory::VolumeShockers);
    assert_eq!(entries[0].signal.symbol.as_deref(), Some("RELIANCE"));
    assert_eq!(entries[1].signal.symbol.as_deref(), Some("TCS"));
    assert_eq!(entries[2].category, IdeaSignalCategory::Breakout52W);
    assert_eq!(entries[3].category, IdeaSignalCategory::MeanReversion);
    assert_eq!(entries[4].signal.symbol.as_deref(), Some("SBIN"));

    // Category cap: only the first `max_categories` categories survive.
    let capped = build_radar_entries(
        vec![
            mk("a", IdeaSignalCategory::VolumeShockers, "RELIANCE"),
            mk("b", IdeaSignalCategory::Breakout52W, "HDFCBANK"),
            mk("c", IdeaSignalCategory::MeanReversion, "ICICIBANK"),
        ],
        2,
        2,
    );
    assert_eq!(capped.len(), 2);
    assert_eq!(capped[1].category, IdeaSignalCategory::Breakout52W);

    // Date formatting: compact "Mon D", verbatim fallback, empty for None.
    assert_eq!(format_idea_date(Some("2026-01-05")), "Jan 5");
    assert_eq!(format_idea_date(Some("not-a-date")), "not-a-date");
    assert_eq!(format_idea_date(None), "");
}

#[test]
fn mini_chart_normalization_axes_and_inr_grouping() {
    let points = vec![
        ChartPoint::Value(100.0),
        ChartPoint::Labeled {
            label: "Jan 5".to_string(),
            value: 104.0,
        },
        ChartPoint::Value(f64::NAN), // filtered
        ChartPoint::Value(110.0),
    ];
    let normalized = normalize_chart_points(&points);
    assert_eq!(normalized.len(), 3);
    assert_eq!(normalized[0].label, "Point 1");
    assert_eq!(normalized[1].label, "Jan 5");
    assert_eq!(normalized[2].label, "Point 4");
    assert_eq!(normalized[2].value, 110.0);

    let y = build_y_axis_labels(100.0, 110.0, |v| format!("{v:.0}"));
    assert_eq!(y, vec!["110", "105", "100"]);
    assert!(build_y_axis_labels(f64::NAN, 110.0, |v| format!("{v}")).is_empty());

    let x = build_x_axis_labels(&normalized);
    assert_eq!(x, vec!["Point 1", "Jan 5", "Point 4"]);
    assert!(build_x_axis_labels(&[]).is_empty());

    // Indian-locale grouping, no fractional digits (reference en-IN default).
    assert_eq!(format_inr_grouping(100.0), "100");
    assert_eq!(format_inr_grouping(42_500.0), "42,500");
    assert_eq!(format_inr_grouping(1_234_567.89), "12,34,568");
    assert_eq!(format_inr_grouping(-42_500.0), "-42,500");
    assert_eq!(format_inr_grouping(f64::NAN), "--");
}

#[test]
fn intel_wire_sentiment_confidence_and_time() {
    assert_eq!(IntelSentiment::Bullish.as_str(), "bullish");
    assert_eq!(IntelSentiment::Bearish.label(), "Bearish");
    assert_eq!(
        IntelSentiment::from_label("NEGATIVE"),
        Some(IntelSentiment::Bearish)
    );
    assert_eq!(IntelSentiment::from_label("meh"), None);

    assert_eq!(format_confidence(0.87), "87%");
    assert_eq!(format_confidence(1.0), "100%");
    assert_eq!(format_confidence(0.004), "0%");
    assert_eq!(format_confidence(f64::NAN), "--");

    // RFC 3339 -> 24h HH:MM; verbatim fallback; empty for None.
    assert_eq!(format_intel_time(Some("2026-01-15T09:05:00Z")), "09:05");
    assert_eq!(
        format_intel_time(Some("2026-01-15T17:30:00+05:30")),
        "17:30"
    );
    assert_eq!(format_intel_time(Some("garbage")), "garbage");
    assert_eq!(format_intel_time(None), "");

    let item = IntelWireItem::new("n1", "Reuters", "RBI holds repo rate steady")
        .with_published_at("2026-01-15T09:05:00Z")
        .with_url("https://example.com/rbi")
        .with_sentiment(IntelSentimentScore::new(IntelSentiment::Neutral, 0.62));
    assert_eq!(item.source, "Reuters");
    assert_eq!(item.sentiment.as_ref().unwrap().confidence_label(), "62%");
}

#[test]
fn results_summary_growth_formatting_and_tones() {
    // Fractions at or below magnitude 1 are scaled by 100 (reference pct()).
    assert_eq!(format_growth_pct(Some(0.123)), "+12.3%");
    assert_eq!(format_growth_pct(Some(-0.041)), "-4.1%");
    assert_eq!(format_growth_pct(Some(12.3)), "+12.3%");
    assert_eq!(format_growth_pct(Some(0.0)), "+0.0%");
    assert_eq!(format_growth_pct(None), "--");
    assert_eq!(format_growth_pct(Some(f64::NAN)), "--");

    assert_eq!(growth_tone(Some(4.2)), GrowthTone::Up);
    assert_eq!(growth_tone(Some(-1.1)), GrowthTone::Down);
    assert_eq!(growth_tone(Some(0.0)), GrowthTone::Neutral);
    assert_eq!(growth_tone(None), GrowthTone::Neutral);

    assert_eq!(format_cr(Some(123_457.0)), "₹1,23,457 Cr");
    assert_eq!(format_cr(None), "--");

    let card = EarningsScorecard::new("RELIANCE", "Reliance Industries", "Q2 FY26")
        .with_revenue(212_340.0, 4.2, 8.9)
        .with_pat(16_575.0, -2.1, 5.4);
    assert_eq!(card.revenue_cr, Some(212_340.0));
    assert_eq!(card.pat_yoy_pct, Some(5.4));
}

#[test]
fn timeline_kinds_tones_and_time_formatting() {
    assert_eq!(TimelineKind::Filing.label(), "FILING");
    assert_eq!(TimelineKind::CorporateAction.label(), "CORPORATE ACTION");
    assert_eq!(TimelineKind::CorporateAction.as_str(), "corporate_action");

    // Sentiment wins over kind.
    assert_eq!(timeline_tone(TimelineKind::News, Some("Bearish")), "down");
    assert_eq!(timeline_tone(TimelineKind::Filing, Some("bull run")), "up");
    // Alert / corporate action default to warning.
    assert_eq!(timeline_tone(TimelineKind::Alert, None), "warning");
    assert_eq!(
        timeline_tone(TimelineKind::CorporateAction, Some("")),
        "warning"
    );
    // Everything else defaults to accent.
    assert_eq!(timeline_tone(TimelineKind::Earnings, None), "accent");
    assert_eq!(timeline_tone(TimelineKind::ModelSignal, None), "accent");

    assert_eq!(
        format_timeline_time(Some("2026-01-15T09:05:00Z")),
        "Jan 15, 09:05"
    );
    assert_eq!(format_timeline_time(Some("garbage")), "garbage");
    assert_eq!(format_timeline_time(None), "Pending");

    let item = TimelineItem::new("t1", TimelineKind::Filing, "Board approves fund raise")
        .with_symbol("TCS")
        .with_timestamp("2026-01-15T09:05:00Z")
        .with_source("NSE")
        .with_url("https://example.com/filing")
        .with_sentiment("Neutral");
    assert_eq!(item.kind, TimelineKind::Filing);
    assert_eq!(item.symbol.as_deref(), Some("TCS"));
}
