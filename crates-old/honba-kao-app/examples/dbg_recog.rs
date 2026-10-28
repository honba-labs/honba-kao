use honba_kao_app::features::simulator::kernel::selection::builtin_template;
use honba_kao_app::features::strategy_studio::kernel::codegen::{generate_code, recognize};

fn main() {
    let graph = builtin_template().0;
    let code = generate_code(&graph, "sha256:t");
    for line in code.lines() {
        let t = line.trim();
        let handled = t == ")"
            || t.is_empty()
            || t.starts_with("GRAPH_HASH = ")
            || t.starts_with(".node(")
            || t.starts_with(".edge(")
            || t == "GRAPH = ("
            || t.starts_with("from ")
            || t.starts_with("_graph.StrategyGraph.build(")
            || t.starts_with(".with_subscriptions(");
        if !handled {
            eprintln!("UNHANDLED: {t:?}");
        }
    }
    let rec = recognize(&code);
    eprintln!(
        "code_only={} graph_ok={}",
        rec.code_only,
        rec.graph.is_some()
    );
}
