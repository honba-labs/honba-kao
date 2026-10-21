//! Parameter inspector (HBD-2026-DES-006 P2) — port of honba-fe
//! algodesigner `components/NodeInspector.tsx`.
//!
//! Three forms, by selection: a node (label, indicator kind, parameters with
//! the catalogue's min/max/step, per-kind enum pickers for ops and sides),
//! an edge (its endpoints and a delete action), or nothing (the
//! strategy-level settings: name, instrument, driving timeframe, warm-up).
//!
//! The form is rebuilt only when its *shape* changes (the selected id, the
//! node's type/indicator, the set of parameter names) — values are read
//! through `prop:value` closures and written on `change`, so editing a
//! value never re-creates the input under the cursor and focus survives a
//! Tab. Out-of-range values are accepted and surface as the validator's
//! `param_out_of_range` issue, the same as fe.

use leptos::prelude::*;
use serde_json::Value;

use crate::features::simulator::kernel::selection::StrategyGraphShape;
use crate::features::strategy_studio::canvas::scene::Selection;
use crate::features::strategy_studio::kernel::graph::{catalogue_entry, ParamSpec, CATALOGUE};
use crate::features::strategy_studio::stores::StudioStore;

/// What the form is built from (its memo key).
#[derive(Clone, Debug, PartialEq)]
enum FormKey {
    Strategy,
    Edge { id: String, from: String, to: String },
    Node { id: String, node_type: String, indicator: Option<String>, params: Vec<String> },
    Missing,
}

/// A select-style parameter's choices, by node type.
fn enum_choices(node_type: &str, name: &str) -> Option<&'static [&'static str]> {
    match (node_type, name) {
        ("condition", "op") => {
            Some(&["crosses_above", "crosses_below", "gt", "gte", "lt", "lte", "eq"])
        }
        ("logic", "op") => Some(&["and", "or", "not"]),
        ("order", "side") => Some(&["buy", "sell"]),
        ("sizing", "mode") => Some(&["fixed_quantity", "percent_equity"]),
        ("transform", "op") => Some(&["pct_change"]),
        _ => None,
    }
}

/// The JSON spelling of an edited value, typed by the catalogue spec when
/// there is one, else by the value it replaces. `None` when unparsable.
pub fn parse_param_input(text: &str, spec: Option<&ParamSpec>, previous: Option<&Value>) -> Option<Value> {
    let text = text.trim();
    let kind = spec.map(|s| s.kind).unwrap_or(match previous {
        Some(Value::Number(n)) if n.is_i64() || n.is_u64() => "int",
        Some(Value::Number(_)) => "float",
        Some(Value::Bool(_)) => "bool",
        _ => "str",
    });
    match kind {
        "int" => text
            .parse::<i64>()
            .ok()
            .map(Value::from)
            .or_else(|| text.parse::<f64>().ok().filter(|f| f.is_finite()).map(|f| Value::from(f.round() as i64))),
        "float" => text
            .parse::<f64>()
            .ok()
            .filter(|f| f.is_finite())
            .map(Value::from),
        "bool" => match text {
            "true" => Some(Value::Bool(true)),
            "false" => Some(Value::Bool(false)),
            _ => None,
        },
        _ => Some(Value::from(text)),
    }
}

fn value_text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn param_text(graph: &StrategyGraphShape, node_id: &str, name: &str) -> String {
    value_text(
        graph
            .nodes
            .iter()
            .find(|n| n.id == node_id)
            .and_then(|n| n.params.get(name)),
    )
}

/// The inspector panel.
#[component]
pub fn Inspector(
    /// The studio store.
    store: StudioStore,
) -> impl IntoView {
    let key = Memo::new(move |_| match store.selection.get() {
        Selection::None => FormKey::Strategy,
        Selection::Edge(id) => store.graph.with(|g| {
            g.edges
                .iter()
                .find(|e| e.id == id)
                .map(|e| FormKey::Edge {
                    id: e.id.clone(),
                    from: format!("{}.{}", e.from_node, e.from_port),
                    to: format!("{}.{}", e.to_node, e.to_port),
                })
                .unwrap_or(FormKey::Missing)
        }),
        Selection::Node(id) => store.graph.with(|g| {
            g.nodes
                .iter()
                .find(|n| n.id == id)
                .map(|n| FormKey::Node {
                    id: n.id.clone(),
                    node_type: n.node_type.clone(),
                    indicator: n.indicator.clone(),
                    params: n.params.keys().cloned().collect(),
                })
                .unwrap_or(FormKey::Missing)
        }),
    });

    view! {
        <section class="studio-inspector" aria-label="Inspector">
            {move || match key.get() {
                FormKey::Strategy => view! { <StrategyForm store=store /> }.into_any(),
                FormKey::Edge { id, from, to } => {
                    view! { <EdgeForm store=store id=id from=from to=to /> }.into_any()
                }
                FormKey::Node { id, node_type, indicator, params } => {
                    view! {
                        <NodeForm
                            store=store
                            id=id
                            node_type=node_type
                            indicator=indicator
                            params=params
                        />
                    }
                        .into_any()
                }
                FormKey::Missing => view! { <div class="studio-inspector__empty">"Selection no longer exists."</div> }.into_any(),
            }}
        </section>
    }
}

#[component]
fn EdgeForm(store: StudioStore, id: String, from: String, to: String) -> impl IntoView {
    view! {
        <div class="studio-inspector__head">
            <span class="studio-inspector__kind">"Edge"</span>
            <span class="studio-inspector__id">{id.clone()}</span>
        </div>
        <dl class="studio-inspector__facts">
            <dt>"From"</dt>
            <dd>{from}</dd>
            <dt>"To"</dt>
            <dd>{to}</dd>
        </dl>
        <div class="studio-inspector__actions">
            <button
                type="button"
                class="studio-btn studio-btn--danger"
                on:click=move |_| store.delete_edge(&id)
            >
                "Delete edge"
            </button>
        </div>
    }
}

#[component]
fn NodeForm(
    store: StudioStore,
    id: String,
    node_type: String,
    indicator: Option<String>,
    params: Vec<String>,
) -> impl IntoView {
    let specs: &'static [ParamSpec] = indicator
        .as_deref()
        .and_then(catalogue_entry)
        .map(|entry| entry.params)
        .unwrap_or(&[]);
    // Catalogue params first (display order), then any other params the
    // node carries (fe templates use names outside the catalogue).
    let mut names: Vec<String> = specs.iter().map(|s| s.name.to_string()).collect();
    for name in &params {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    // A condition may compare against a threshold instead of a right input.
    let offers_threshold = node_type == "condition" && !params.iter().any(|p| p == "threshold");

    let label_id = id.clone();
    let label_value_id = id.clone();
    let delete_id = id.clone();
    let threshold_id = id.clone();
    let summary = indicator
        .as_deref()
        .and_then(catalogue_entry)
        .map(|entry| entry.summary)
        .unwrap_or("");

    let indicator_picker = indicator.clone().map(|current| {
        let pick_id = id.clone();
        view! {
            <label class="studio-field">
                <span>"Indicator"</span>
                <select on:change=move |ev| store.set_indicator(&pick_id, &event_target_value(&ev))>
                    {CATALOGUE
                        .iter()
                        .map(|entry| {
                            view! {
                                <option value=entry.kind selected={entry.kind == current.as_str()}>
                                    {entry.kind.to_uppercase()}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
            </label>
        }
    });

    let fields = names
        .into_iter()
        .map(|name| {
            let spec = specs.iter().find(|s| s.name == name).copied();
            let read_id = id.clone();
            let read_name = name.clone();
            let write_id = id.clone();
            let write_name = name.clone();
            let remove_id = id.clone();
            let remove_name = name.clone();
            let removable = spec.is_none() && name == "threshold";
            let description = spec.map(|s| s.description).unwrap_or("");
            let value = move || store.graph.with(|g| param_text(g, &read_id, &read_name));
            let control = match enum_choices(&node_type, &name) {
                Some(choices) => {
                    let current = store
                        .graph
                        .with_untracked(|g| param_text(g, &id, &name));
                    view! {
                        <select on:change=move |ev| {
                            store.set_param(&write_id, &write_name, Value::from(event_target_value(&ev)));
                        }>
                            {choices
                                .iter()
                                .map(|choice| {
                                    view! {
                                        <option value={*choice} selected={*choice == current.as_str()}>
                                            {*choice}
                                        </option>
                                    }
                                })
                                .collect_view()}
                        </select>
                    }
                        .into_any()
                }
                None => {
                    let numeric = spec.is_some_and(|s| s.min.is_some() || s.max.is_some())
                        || store.graph.with_untracked(|g| {
                            g.nodes
                                .iter()
                                .find(|n| n.id == id)
                                .and_then(|n| n.params.get(&name))
                                .is_some_and(Value::is_number)
                        });
                    let min = spec.and_then(|s| s.min).map(|v| v.to_string()).unwrap_or_default();
                    let max = spec.and_then(|s| s.max).map(|v| v.to_string()).unwrap_or_default();
                    let step = spec
                        .and_then(|s| s.step)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "any".to_string());
                    view! {
                        <input
                            type={if numeric { "number" } else { "text" }}
                            min=min
                            max=max
                            step=step
                            prop:value=value
                            on:change=move |ev| {
                                let text = event_target_value(&ev);
                                let previous = store
                                    .graph
                                    .with_untracked(|g| {
                                        g.nodes
                                            .iter()
                                            .find(|n| n.id == write_id)
                                            .and_then(|n| n.params.get(&write_name).cloned())
                                    });
                                match parse_param_input(&text, spec.as_ref(), previous.as_ref()) {
                                    Some(parsed) => store.set_param(&write_id, &write_name, parsed),
                                    None => {
                                        store
                                            .notice
                                            .set(Some(format!("`{write_name}` needs a number")));
                                    }
                                }
                            }
                        />
                    }
                        .into_any()
                }
            };
            view! {
                <label class="studio-field" title=description>
                    <span>
                        {name.clone()}
                        {removable
                            .then(|| {
                                view! {
                                    <button
                                        type="button"
                                        class="studio-field__remove"
                                        title="Remove the threshold"
                                        on:click=move |ev| {
                                            ev.prevent_default();
                                            store.remove_param(&remove_id, &remove_name);
                                        }
                                    >
                                        "×"
                                    </button>
                                }
                            })}
                    </span>
                    {control}
                </label>
            }
        })
        .collect_view();

    view! {
        <div class="studio-inspector__head">
            <span class="studio-inspector__kind">{node_type.clone()}</span>
            <span class="studio-inspector__id">{id.clone()}</span>
        </div>
        {(!summary.is_empty()).then(|| view! { <p class="studio-inspector__summary">{summary}</p> })}
        <label class="studio-field">
            <span>"Label"</span>
            <input
                type="text"
                prop:value=move || {
                    store
                        .graph
                        .with(|g| {
                            g.nodes
                                .iter()
                                .find(|n| n.id == label_value_id)
                                .and_then(|n| n.label.clone())
                                .unwrap_or_default()
                        })
                }
                on:change=move |ev| store.set_label(&label_id, &event_target_value(&ev))
            />
        </label>
        {indicator_picker}
        {fields}
        {offers_threshold
            .then(|| {
                view! {
                    <button
                        type="button"
                        class="studio-btn studio-btn--ghost"
                        title="Compare the left input against a constant instead of a right input"
                        on:click=move |_| store.set_param(&threshold_id, "threshold", Value::from(0))
                    >
                        "+ Threshold"
                    </button>
                }
            })}
        <div class="studio-inspector__actions">
            <button
                type="button"
                class="studio-btn studio-btn--danger"
                on:click=move |_| store.delete_node(&delete_id)
            >
                "Delete node"
            </button>
        </div>
    }
}

const AGGREGATIONS: &[&str] = &["second", "minute", "hour", "day", "week", "month"];

#[component]
fn StrategyForm(store: StudioStore) -> impl IntoView {
    let graph = store.graph;
    let symbol = move || graph.with(|g| g.universe_explicit.first().map(|(s, _)| s.clone()).unwrap_or_default());
    let exchange = move || graph.with(|g| g.universe_explicit.first().map(|(_, e)| e.clone()).unwrap_or_default());
    let current_aggregation = graph.with_untracked(|g| g.driving_aggregation.clone());
    view! {
        <div class="studio-inspector__head">
            <span class="studio-inspector__kind">"Strategy"</span>
            <span class="studio-inspector__id">"select a node or edge to edit it"</span>
        </div>
        <label class="studio-field">
            <span>"Name"</span>
            <input
                type="text"
                prop:value=move || graph.with(|g| g.name.clone())
                on:change=move |ev| store.set_name(&event_target_value(&ev))
            />
        </label>
        <div class="studio-field-row">
            <label class="studio-field">
                <span>"Symbol"</span>
                <input
                    type="text"
                    prop:value=symbol
                    on:change=move |ev| {
                        let exchange = graph
                            .with_untracked(|g| {
                                g.universe_explicit.first().map(|(_, e)| e.clone()).unwrap_or_default()
                            });
                        store.set_instrument(&event_target_value(&ev), &exchange);
                    }
                />
            </label>
            <label class="studio-field">
                <span>"Exchange"</span>
                <input
                    type="text"
                    prop:value=exchange
                    on:change=move |ev| {
                        let symbol = graph
                            .with_untracked(|g| {
                                g.universe_explicit.first().map(|(s, _)| s.clone()).unwrap_or_default()
                            });
                        store.set_instrument(&symbol, &event_target_value(&ev));
                    }
                />
            </label>
        </div>
        <div class="studio-field-row">
            <label class="studio-field">
                <span>"Interval"</span>
                <input
                    type="number"
                    min="1"
                    step="1"
                    prop:value=move || graph.with(|g| g.driving_interval.to_string())
                    on:change=move |ev| {
                        let aggregation = graph.with_untracked(|g| g.driving_aggregation.clone());
                        match event_target_value(&ev).trim().parse::<u32>() {
                            Ok(interval) => store.set_timeframe(interval, &aggregation),
                            Err(_) => store.notice.set(Some("interval must be a whole number".to_string())),
                        }
                    }
                />
            </label>
            <label class="studio-field">
                <span>"Unit"</span>
                <select on:change=move |ev| {
                    let interval = graph.with_untracked(|g| g.driving_interval);
                    store.set_timeframe(interval, &event_target_value(&ev));
                }>
                    {AGGREGATIONS
                        .iter()
                        .map(|unit| {
                            view! {
                                <option value={*unit} selected={*unit == current_aggregation.as_str()}>
                                    {*unit}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
            </label>
        </div>
        <label class="studio-field">
            <span>"Warm-up bars"</span>
            <input
                type="number"
                min="0"
                step="1"
                prop:value=move || graph.with(|g| g.warmup_bars.to_string())
                on:change=move |ev| {
                    match event_target_value(&ev).trim().parse::<u32>() {
                        Ok(bars) => store.set_warmup(bars),
                        Err(_) => store.notice.set(Some("warm-up must be a whole number".to_string())),
                    }
                }
            />
        </label>
    }
}
