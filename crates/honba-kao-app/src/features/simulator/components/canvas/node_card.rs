// Generic node card: 200–300px, category top-border color
// (`--c-*` tokens), issue badge, per-kind body chips, ports as named
// rows with typed color dots.

use leptos::prelude::*;

/// Category colors (match fe's `--c-*` token set).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeCategory {
    Indicator,
    Condition,
    Transform,
    Sizing,
    Risk,
    Order,
    Benchmark,
    Universe,
    Logic,
}

impl NodeCategory {
    fn color_token(&self) -> &'static str {
        match self {
            NodeCategory::Indicator => "--c-indicator",
            NodeCategory::Condition => "--c-condition",
            NodeCategory::Transform => "--c-transform",
            NodeCategory::Sizing => "--c-sizing",
            NodeCategory::Risk => "--c-risk",
            NodeCategory::Order => "--c-order",
            NodeCategory::Benchmark => "--c-benchmark",
            NodeCategory::Universe => "--c-universe",
            NodeCategory::Logic => "--c-logic",
        }
    }

    fn badge_text(&self) -> &'static str {
        match self {
            NodeCategory::Indicator => "INDICATOR",
            NodeCategory::Condition => "CONDITION",
            NodeCategory::Transform => "TRANSFORM",
            NodeCategory::Sizing => "SIZING",
            NodeCategory::Risk => "RISK",
            NodeCategory::Order => "ORDER",
            NodeCategory::Benchmark => "BENCHMARK",
            NodeCategory::Universe => "UNIVERSE",
            NodeCategory::Logic => "LOGIC",
        }
    }
}

/// One port row: name + typed color dot.
fn port_row(name: &'static str, port_kind: &'static str) -> IntoView {
    let dot_color = NodeCategory_from_kind(port_kind).color_token();
    view! {
        <div>
            <span>{name}</span>
            <span
                style=format!("color: var({}); width: 8px; height: 8px; border-radius: 4px;", dot_color).into_any()>
            </span>
        </div>
    }
}

fn NodeCategory_from_kind(kind: &str) -> NodeCategory {
    match kind {
        "series" => NodeCategory::Indicator,
        "bool" => NodeCategory::Condition,
        "size" => NodeCategory::Sizing,
        "order" => NodeCategory::Order,
        _ => NodeCategory::Condition,
    }
}

/// Generic node card: 200–300px, category top-border color
// (`--c-*` tokens), issue badge, per-kind body chips, ports as named
// rows with typed color dots.
#[component]
pub fn NodeCard {
    #[prop(into)]
    category: NodeCategory,
    #[prop(into, optional)]
    issue_count: Option<usize>,
    #[prop(into, optional)]
    ports: Option<Vec<(&'static str, &'static str)>>,

    let cat = category;
    let issue_count = issue_count.unwrap_or(0);
    let ports = ports.unwrap_or_default();

    view! {
        <div style=format!("border-top: 4px solid var({});", cat.color_token())>
            <div>
                <span>{cat.badge_text()}</span>
                <span>{issue_count}</span>
            </div>
            <div>
                {ports.iter().map(|(name, kind)| {
                    port_row(name, kind)
                }).collect_view()}
            </div>
        </div>
    }
}