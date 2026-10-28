//! AiInsightCard — on-demand AI market outlook panel.
//!
//! Port of OpenTerminalUI's `components/terminal/AiInsightCard.tsx`. Lazy by
//! design: nothing runs until the user clicks **Generate**, because local LLM
//! inference is slow. The Mission Control page feeds the macro regime
//! classification and institutional-flows summary through [`InsightData`]
//! sections (plan-1.md §3 2.1).
//!
//! The fetcher is a synchronous [`Callback`] so the card stays testable on the
//! host target; the async AI backend resolves its future before invoking it.

use leptos::prelude::*;

use honba_kao_ui_core::{HfButton, HfButtonSize, HfButtonVariant};

/// Tone of an insight section — drives the section title and bullet color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InsightTone {
    Positive,
    #[default]
    Neutral,
    Negative,
}

impl InsightTone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Neutral => "neutral",
            Self::Negative => "negative",
        }
    }
}

/// One titled block of insight bullet points.
#[derive(Debug, Clone, PartialEq)]
pub struct InsightSection {
    pub title: String,
    pub tone: InsightTone,
    pub points: Vec<String>,
}

impl InsightSection {
    pub fn new(title: impl Into<String>, tone: InsightTone, points: Vec<String>) -> Self {
        Self {
            title: title.into(),
            tone,
            points,
        }
    }
}

/// Resolved AI analysis payload.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InsightData {
    /// Serving engine identifier, e.g. `"lm-studio"` or `"unavailable"`.
    pub engine: String,
    /// Concrete model name reported by the engine, when known.
    pub model: Option<String>,
    /// One-paragraph macro summary.
    pub summary: Option<String>,
    /// Titled sections (regime classification, institutional flows, …).
    pub sections: Vec<InsightSection>,
    /// RFC 3339 timestamp of generation, when the engine reports one.
    pub generated_at: Option<String>,
}

impl InsightData {
    pub fn new(engine: impl Into<String>) -> Self {
        Self {
            engine: engine.into(),
            ..Self::default()
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn with_sections(mut self, sections: Vec<InsightSection>) -> Self {
        self.sections = sections;
        self
    }
}

/// Lifecycle of the on-demand analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiInsightStatus {
    #[default]
    Idle,
    Loading,
    Done,
    Error,
}

impl AiInsightStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Loading => "loading",
            Self::Done => "done",
            Self::Error => "error",
        }
    }
}

/// Any engine that answered is live (LM Studio / vLLM or OpenRouter); the
/// label names the actual model rather than assuming one.
pub fn engine_label(engine: &str, model: Option<&str>) -> String {
    let live = !engine.is_empty() && engine != "unavailable";
    if live {
        model
            .filter(|m| !m.trim().is_empty())
            .unwrap_or(engine)
            .to_string()
    } else {
        "AI unavailable".to_string()
    }
}

/// Resolves the insight. Invoked only when the user clicks Generate.
pub type InsightFetcher = Callback<(), Result<InsightData, String>>;

#[component]
pub fn AiInsightCard(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: String,
    fetcher: InsightFetcher,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let (status, set_status) = signal(AiInsightStatus::Idle);
    let (data, set_data) = signal(Option::<InsightData>::None);

    let run = Callback::new(move |_: web_sys::MouseEvent| {
        set_status.set(AiInsightStatus::Loading);
        set_data.set(None);
        match fetcher.run(()) {
            Ok(result) => {
                set_data.set(Some(result));
                set_status.set(AiInsightStatus::Done);
            }
            Err(_) => set_status.set(AiInsightStatus::Error),
        }
    });

    let busy = Signal::derive(move || status.get() == AiInsightStatus::Loading);

    let button_label = move || match status.get() {
        AiInsightStatus::Loading => "Analyzing…",
        AiInsightStatus::Idle => "Generate",
        AiInsightStatus::Done | AiInsightStatus::Error => "Regenerate",
    };

    let engine_chip = move || {
        if status.get() != AiInsightStatus::Done {
            return None;
        }
        let d = data.get()?;
        let live = !d.engine.is_empty() && d.engine != "unavailable";
        Some(view! {
            <span
                class="ot-home-ai-insight-engine"
                data-live=if live { "true" } else { "false" }
            >
                {engine_label(&d.engine, d.model.as_deref())}
            </span>
        })
    };

    let idle_hint = move || {
        (status.get() == AiInsightStatus::Idle).then(|| {
            view! {
                <p class="ot-home-ai-insight-hint">
                    "Generated by the configured AI model — analysis can take a moment."
                </p>
            }
        })
    };

    let loading_view = move || {
        (status.get() == AiInsightStatus::Loading).then(|| {
            view! {
                <div class="ot-home-ai-insight-loading" aria-busy="true" role="status">
                    <span class="ot-home-ai-insight-loading-label">"Generating analysis…"</span>
                    <span class="ot-home-ai-insight-loading-hint">
                        "A local model usually takes 5–20 seconds."
                    </span>
                </div>
            }
        })
    };

    let error_view = move || {
        (status.get() == AiInsightStatus::Error).then(|| {
            view! {
                <p class="ot-home-ai-insight-error" role="alert">
                    "Could not generate AI analysis right now. Please retry in a moment."
                </p>
            }
        })
    };

    let body_view = move || {
        if status.get() != AiInsightStatus::Done {
            return None;
        }
        let d = data.get()?;
        let engine_unavailable = d.engine.is_empty() || d.engine == "unavailable";
        if engine_unavailable && d.sections.is_empty() {
            return Some(
                view! {
                    <p class="ot-home-ai-insight-hint">
                        "The AI model is temporarily unavailable. Click Regenerate to retry."
                    </p>
                }
                .into_any(),
            );
        }
        let summary = d.summary.clone().map(|s| {
            view! { <p class="ot-home-ai-insight-summary">{s}</p> }
        });
        let sections = d
            .sections
            .iter()
            .map(|section| {
                let tone = section.tone.as_str();
                let points = section
                    .points
                    .iter()
                    .map(|point| {
                        view! {
                            <li class="ot-home-ai-insight-point">
                                <span class="ot-home-ai-insight-point-marker" data-tone=tone>
                                    "▸"
                                </span>
                                <span>{point.clone()}</span>
                            </li>
                        }
                    })
                    .collect_view();
                view! {
                    <div class="ot-home-ai-insight-section">
                        <h4 class="ot-home-ai-insight-section-title" data-tone=tone>
                            {section.title.clone()}
                        </h4>
                        <ul class="ot-home-ai-insight-list">{points}</ul>
                    </div>
                }
            })
            .collect_view();
        Some(
            view! {
                <div class="ot-home-ai-insight-body">
                    {summary}
                    {sections}
                </div>
            }
            .into_any(),
        )
    };

    let card_class = format!("ot-home-ai-insight {class}").trim().to_string();
    let has_description = !description.is_empty();

    view! {
        <section class=card_class data-status=status.get().as_str() aria-label=title.clone()>
            <header class="ot-home-ai-insight-header">
                <div>
                    <h3 class="ot-home-ai-insight-title">{title.clone()}</h3>
                    <Show when=move || has_description>
                        <p class="ot-home-ai-insight-description">{description.clone()}</p>
                    </Show>
                </div>
                <div class="ot-home-ai-insight-actions">
                    {engine_chip}
                    <HfButton
                        variant=HfButtonVariant::Ghost
                        size=HfButtonSize::Small
                        disabled=busy
                        on_click=run
                        class="ot-home-ai-insight-generate"
                    >
                        {button_label}
                    </HfButton>
                </div>
            </header>

            {idle_hint}
            {loading_view}
            {error_view}
            {body_view}
        </section>
    }
}
