//! Strategy Studio P2 (HBD-2026-DES-006) — the node editor's pure layers:
//! templates, graph edits, splice filtering, palette payloads, geometry and
//! the single render-pass scene. Lives in `tests/` per AGENTS.md §1.1; it
//! exercises only the crate's public API (no DOM).

use std::collections::BTreeMap;

use honba_kao_app::features::simulator::kernel::selection::StrategyGraphShape;
use honba_kao_app::features::strategy_studio::canvas::edge::{
    splice_node, splice_options_for, SPLICE_OPTIONS,
};
use honba_kao_app::features::strategy_studio::canvas::geometry::{
    auto_layout, smoothstep_path, snap, snap_position, Position, Viewport, GRID,
};
use honba_kao_app::features::strategy_studio::canvas::inspector::parse_param_input;
use honba_kao_app::features::strategy_studio::canvas::palette::{
    palette_items, palette_node, palette_payload, parse_palette_payload,
};
use honba_kao_app::features::strategy_studio::canvas::scene::{build_scene, Selection};
use honba_kao_app::features::strategy_studio::kernel::edit::{self, EditError};
use honba_kao_app::features::strategy_studio::kernel::graph::{
    catalogue_entry, default_node, graph_hash, graph_issues, PortKind,
};
use honba_kao_app::features::strategy_studio::kernel::templates::{
    blank_graph, library_template, strategy_templates, template_by_name,
};

fn template(name: &str) -> StrategyGraphShape {
    template_by_name(name).expect("template exists").graph
}

fn port_mismatches(graph: &StrategyGraphShape) -> usize {
    graph_issues(graph)
        .iter()
        .filter(|i| i.code == "port_mismatch")
        .count()
}

// -- Templates ---------------------------------------------------------------

#[test]
fn the_six_fe_templates_port_in_fe_order() {
    let names: Vec<&str> = strategy_templates().iter().map(|t| t.name).collect();
    assert_eq!(
        names,
        ["sma_cross", "ema_trend", "rsi_mr", "macd_rsi", "bb_breakout", "atr_trailing"]
    );
    for t in strategy_templates() {
        assert_eq!(t.graph.name, t.name);
        assert_eq!(port_mismatches(&t.graph), 0, "{} ports type-check", t.name);
        assert!(
            !graph_issues(&t.graph).iter().any(|i| i.code == "cycle_detected"),
            "{} is acyclic",
            t.name
        );
    }
}

#[test]
fn templates_round_trip_through_json_with_a_stable_hash() {
    for t in strategy_templates() {
        let json = serde_json::to_string(&t.graph).unwrap();
        let back: StrategyGraphShape = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t.graph, "{}", t.name);
        assert_eq!(graph_hash(&back), graph_hash(&t.graph), "{}", t.name);
    }
}

#[test]
fn the_blank_graph_and_library_mapping_are_honest() {
    let blank = blank_graph();
    assert_eq!(blank.nodes.len(), 1);
    assert!(graph_issues(&blank).iter().any(|i| i.code == "no_order"));
    assert_eq!(library_template("trend-ema"), Some("ema_trend"));
    assert_eq!(library_template("vol-breakout"), None);
    for id in ["trend-ema", "bollinger-squeeze"] {
        assert!(template_by_name(library_template(id).unwrap()).is_some());
    }
}

// -- Render pass -----------------------------------------------------------

#[test]
fn every_template_renders_every_node_and_edge_in_one_pass() {
    for t in strategy_templates() {
        let positions = auto_layout(&t.graph);
        assert_eq!(positions.len(), t.graph.nodes.len(), "{}", t.name);
        for p in positions.values() {
            assert_eq!(*p, snap_position(*p), "{} positions are on the grid", t.name);
        }
        let scene = build_scene(&t.graph, &positions, &Selection::None, &graph_issues(&t.graph));
        assert_eq!(scene.nodes.len(), t.graph.nodes.len(), "{}", t.name);
        assert_eq!(scene.edges.len(), t.graph.edges.len(), "{}", t.name);
        assert!(scene.edges.iter().all(|e| e.kind.is_some() && e.d.starts_with('M')));
    }
}

#[test]
fn auto_layout_places_downstream_nodes_in_later_columns() {
    let graph = template("sma_cross");
    let positions = auto_layout(&graph);
    assert!(positions["n1"].x < positions["f"].x);
    assert!(positions["f"].x < positions["c"].x);
    assert!(positions["c"].x < positions["o"].x);
    // Siblings share a column without overlapping.
    assert_eq!(positions["f"].x, positions["s"].x);
    assert_ne!(positions["f"].y, positions["s"].y);
}

#[test]
fn missing_positions_fall_back_to_the_layout_and_selection_is_flagged() {
    let graph = template("macd_rsi");
    let scene = build_scene(&graph, &BTreeMap::new(), &Selection::Node("and".into()), &[]);
    let and = scene.nodes.iter().find(|n| n.id == "and").unwrap();
    assert!(and.selected);
    assert!(and.x > 0.0);
    assert_eq!(scene.nodes.iter().filter(|n| n.selected).count(), 1);
    // MACD exposes its three named outputs as port rows.
    let macd = scene.nodes.iter().find(|n| n.id == "macd").unwrap();
    let outs: Vec<&str> = macd.outputs.iter().map(|p| p.id).collect();
    assert_eq!(outs, ["macd", "signal", "histogram"]);
}

#[test]
fn node_issues_land_on_their_card() {
    let mut graph = template("sma_cross");
    graph.nodes.iter_mut().find(|n| n.id == "f").unwrap().params.insert("period".into(), 0.into());
    let issues = graph_issues(&graph);
    let scene = build_scene(&graph, &auto_layout(&graph), &Selection::None, &issues);
    let fast = scene.nodes.iter().find(|n| n.id == "f").unwrap();
    assert_eq!(fast.issues.len(), 1);
}

// -- Edits -----------------------------------------------------------------

#[test]
fn connect_refuses_mismatches_self_loops_and_cycles() {
    let mut graph = template("sma_cross");
    let before = graph.clone();
    assert_eq!(
        edit::connect(&mut graph, "c", "signal", "f", "input"),
        Err(EditError::KindMismatch { from: PortKind::Bool, to: PortKind::Series })
    );
    assert_eq!(edit::connect(&mut graph, "f", "value", "f", "input"), Err(EditError::SelfLoop));
    assert!(matches!(
        edit::connect(&mut graph, "f", "nope", "c", "left"),
        Err(EditError::UnknownPort { .. })
    ));
    assert_eq!(graph, before, "a refused edit leaves the graph untouched");

    // A cycle: indicator → indicator chain closed back on itself.
    let mut chain = template("sma_cross");
    let a = edit::add_node(&mut chain, default_node("", "indicator", Some("ema")));
    edit::connect(&mut chain, "f", "value", &a, "input").unwrap();
    let snapshot = chain.clone();
    assert_eq!(edit::connect(&mut chain, &a, "value", "f", "input"), Err(EditError::Cycle));
    assert_eq!(chain, snapshot);
}

#[test]
fn a_new_source_replaces_the_old_edge_into_an_input() {
    let mut graph = template("sma_cross");
    let id = edit::connect(&mut graph, "s", "value", "c", "left").unwrap();
    let into_left: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.to_node == "c" && e.to_port == "left")
        .collect();
    assert_eq!(into_left.len(), 1);
    assert_eq!(into_left[0].id, id);
    assert_eq!(id, "e_s_value_c_left");
    // Re-connecting the identical edge is a no-op.
    let edges = graph.edges.len();
    assert_eq!(edit::connect(&mut graph, "s", "value", "c", "left").unwrap(), id);
    assert_eq!(graph.edges.len(), edges);
}

#[test]
fn add_node_uniquifies_ids_and_remove_node_drops_incident_edges() {
    let mut graph = template("sma_cross");
    let first = edit::add_node(&mut graph, default_node("", "indicator", Some("sma")));
    let second = edit::add_node(&mut graph, default_node("", "indicator", Some("sma")));
    assert_eq!(first, "sma");
    assert_eq!(second, "sma_1");
    assert!(edit::remove_node(&mut graph, "c"));
    assert!(graph.edges.iter().all(|e| e.from_node != "c" && e.to_node != "c"));
    assert!(!edit::remove_node(&mut graph, "c"));
}

#[test]
fn switching_an_indicator_resets_params_and_drops_vanished_outputs() {
    let mut graph = template("macd_rsi");
    assert!(edit::set_indicator(&mut graph, "macd", "sma"));
    let macd = graph.nodes.iter().find(|n| n.id == "macd").unwrap();
    assert_eq!(macd.indicator.as_deref(), Some("sma"));
    assert_eq!(macd.params.keys().collect::<Vec<_>>(), ["period"]);
    assert!(graph.edges.iter().all(|e| e.from_node != "macd" || e.from_port == "value"));
    assert_eq!(port_mismatches(&graph), 0);
}

// -- Splice ----------------------------------------------------------------

#[test]
fn splice_options_type_check_on_both_sides() {
    let graph = template("sma_cross");
    // bool edge into order.signal: gates fit; sizing/orders would orphan it.
    let labels: Vec<&str> = splice_options_for(&graph, "e5").iter().map(|o| o.label).collect();
    assert_eq!(labels, ["AND Gate", "OR Gate", "Risk Gate"]);
    // series edge into an indicator input: the series options.
    let labels: Vec<&str> = splice_options_for(&graph, "e1").iter().map(|o| o.label).collect();
    assert_eq!(labels, ["SMA Indicator", "EMA Indicator", "RSI Indicator", "Pct Change"]);
    assert!(splice_options_for(&graph, "nope").is_empty());
}

#[test]
fn every_template_edge_splices_without_orphan_handles() {
    for t in strategy_templates() {
        for edge in &t.graph.edges {
            let options = splice_options_for(&t.graph, &edge.id);
            // Series and bool edges always offer something; fe's menu has
            // nothing that passes a `size` through (the menu says so).
            let kind = edit::output_kind(&t.graph, &edge.from_node, &edge.from_port);
            if matches!(kind, Some(PortKind::Series | PortKind::Bool)) {
                assert!(!options.is_empty(), "{} edge {} offers a splice", t.name, edge.id);
            } else {
                assert!(options.is_empty(), "{} edge {}", t.name, edge.id);
            }
            for option in options {
                let mut graph = t.graph.clone();
                let new_id = edit::splice_edge(
                    &mut graph,
                    &edge.id,
                    splice_node(&option),
                    option.in_port,
                    option.out_port,
                )
                .unwrap_or_else(|e| panic!("{} {} {}: {}", t.name, edge.id, option.label, e.message()));
                assert_eq!(graph.nodes.len(), t.graph.nodes.len() + 1);
                assert_eq!(graph.edges.len(), t.graph.edges.len() + 1);
                assert!(!graph.edges.iter().any(|e| e.id == edge.id));
                assert!(graph.edges.iter().any(|e| e.to_node == new_id));
                assert!(graph.edges.iter().any(|e| e.from_node == new_id));
                assert_eq!(port_mismatches(&graph), 0, "{} {} {}", t.name, edge.id, option.label);
                let scene = build_scene(&graph, &auto_layout(&graph), &Selection::None, &[]);
                assert_eq!(scene.edges.len(), graph.edges.len());
            }
        }
    }
}

#[test]
fn a_splice_that_does_not_fit_is_refused() {
    let mut graph = template("sma_cross");
    let before = graph.clone();
    let buy = SPLICE_OPTIONS.iter().find(|o| o.label == "Buy Order").unwrap();
    assert!(matches!(
        edit::splice_edge(&mut graph, "e5", splice_node(buy), buy.in_port, buy.out_port),
        Err(EditError::KindMismatch { .. })
    ));
    assert_eq!(graph, before);
}

// -- Palette ---------------------------------------------------------------

#[test]
fn palette_payloads_round_trip_and_reject_foreign_drops() {
    for item in palette_items() {
        let payload = parse_palette_payload(&palette_payload(&item)).expect("own payload parses");
        let node = palette_node(&payload);
        assert_eq!(node.node_type, item.node_type);
        assert!(node.id.is_empty(), "the store assigns ids");
        if let Some(kind) = item.indicator {
            let entry = catalogue_entry(kind).unwrap();
            assert_eq!(node.params.len(), entry.params.len());
        }
    }
    assert!(parse_palette_payload("").is_none());
    assert!(parse_palette_payload("not json").is_none());
    assert!(parse_palette_payload(r#"{"node_type":"indicator"}"#).is_none());
    assert!(parse_palette_payload(r#"{"node_type":"indicator","indicator":"psar"}"#).is_none());
    assert!(parse_palette_payload(r#"{"node_type":"order","indicator":"sma"}"#).is_none());
    assert!(parse_palette_payload(r#"{"node_type":"exec","indicator":null}"#).is_none());
    assert!(parse_palette_payload(r#"{"node_type":"order","code":"rm -rf"}"#).is_none());
    assert!(parse_palette_payload(r#"{"node_type":"order"}"#).is_some());
}

#[test]
fn the_palette_offers_nine_indicators_and_the_structural_types() {
    let items = palette_items();
    assert_eq!(items.iter().filter(|i| i.indicator.is_some()).count(), 9);
    assert!(items.iter().any(|i| i.node_type == "order"));
    assert!(items.iter().any(|i| i.node_type == "universe"));
}

// -- Geometry --------------------------------------------------------------

#[test]
fn snapping_is_to_the_16px_grid() {
    assert_eq!(GRID, 16.0);
    assert_eq!(snap(7.9), 0.0);
    assert_eq!(snap(8.1), 16.0);
    assert_eq!(snap(-9.0), -16.0);
    assert_eq!(snap_position(Position::new(33.0, 47.0)), Position::new(32.0, 48.0));
}

#[test]
fn wheel_zoom_keeps_the_point_under_the_cursor() {
    let vp = Viewport { x: 40.0, y: -20.0, zoom: 1.0 };
    let (wx, wy) = vp.to_world(300.0, 200.0);
    let zoomed = vp.zoom_at(300.0, 200.0, 1.5);
    let (sx, sy) = zoomed.to_screen(wx, wy);
    assert!((sx - 300.0).abs() < 1e-9 && (sy - 200.0).abs() < 1e-9);
    assert_eq!(vp.zoom_at(0.0, 0.0, 1000.0).zoom, 2.5);
    assert_eq!(vp.zoom_at(0.0, 0.0, 0.0001).zoom, 0.25);
}

#[test]
fn smoothstep_paths_route_forward_and_backward() {
    let (d, mx, my) = smoothstep_path((0.0, 0.0), (200.0, 100.0));
    assert!(d.starts_with("M 0.0,0.0") && d.ends_with("L 200.0,100.0"));
    assert_eq!((mx, my), (100.0, 50.0));
    let (straight, _, _) = smoothstep_path((0.0, 10.0), (100.0, 10.0));
    assert_eq!(straight, "M 0.0,10.0 L 100.0,10.0");
    // Backward edges step out right and in from the left.
    let (back, _, _) = smoothstep_path((300.0, 0.0), (0.0, 100.0));
    assert!(back.contains("320.0") && back.ends_with("L 0.0,100.0"));
}

#[test]
fn fit_frames_all_cards() {
    let graph = template("atr_trailing");
    let positions = auto_layout(&graph);
    let scene = build_scene(&graph, &positions, &Selection::None, &[]);
    let vp = Viewport::fit(&scene.positions(), &scene.heights(), 800.0, 500.0);
    for node in &scene.nodes {
        let (x0, y0) = vp.to_screen(node.x, node.y);
        let (x1, y1) = vp.to_screen(node.x + node.width, node.y + node.height);
        assert!(x0 >= 0.0 && y0 >= 0.0 && x1 <= 800.0 && y1 <= 500.0, "{} fits", node.id);
    }
    assert!(vp.zoom <= 1.0);
}

// -- View state stays out of the graph ------------------------------------

#[test]
fn positions_never_enter_the_graph_document() {
    let graph = template("sma_cross");
    let _positions = auto_layout(&graph);
    let json = serde_json::to_value(&graph).unwrap();
    let keys: Vec<&String> = json.as_object().unwrap().keys().collect();
    assert!(!keys.iter().any(|k| k.contains("position")));
    for node in json["nodes"].as_array().unwrap() {
        assert!(node.get("x").is_none() && node.get("position").is_none());
    }
}

// -- Inspector parsing -----------------------------------------------------

#[test]
fn inspector_values_are_typed_by_spec_then_by_previous_value() {
    let sma = catalogue_entry("sma").unwrap();
    let period = &sma.params[0];
    assert_eq!(parse_param_input("21", Some(period), None), Some(21.into()));
    assert_eq!(parse_param_input("21.6", Some(period), None), Some(22.into()));
    assert_eq!(parse_param_input("abc", Some(period), None), None);
    let k = &catalogue_entry("bollinger").unwrap().params[1];
    assert_eq!(parse_param_input("2.5", Some(k), None), Some(2.5.into()));
    assert_eq!(parse_param_input("0.04", None, Some(&0.03.into())), Some(0.04.into()));
    assert_eq!(parse_param_input("70", None, Some(&30.into())), Some(70.into()));
    assert_eq!(parse_param_input("sell", None, Some(&"buy".into())), Some("sell".into()));
}
