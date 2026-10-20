//! Strategy manifest + compile seam (HBD-2026-DES-005 P4).
//!
//! The run-facing slice of honba-fe `features/algodesigner/{manifest.ts,
//! compile.ts}`: the graph authored in the strategy studio is *converted to a
//! compile manifest*, posted, and the returned content id becomes the run's
//! `strategy` — fe's "compile-before-run" marry (§8.2 contract gap).
//!
//! Two adaptations, both documented:
//! - fe's manifest was `serde`-loose and the fe backend accepted the graph
//!   doc itself; honba-ya verifies a **typed** honba-strategy manifest
//!   (`deny_unknown_fields`), so [`strategy_manifest_value`] maps the graph
//!   onto *that* shape. Same source values, destination schema.
//! - `source_hash`: fe left the digests to the backend. honba-ya derives a
//!   strategy's identity from it, so the digest is content-relevant (FNV-1a
//!   over the canonical graph JSON) — two different graphs never share an
//!   identity. It is explicitly not a cryptographic digest of source code:
//!   nothing on this side compiles source until the studio lands (P8).

use honba_kao_client::{wire::CompiledStrategy, ClientError};
use serde_json::{json, Value};

use crate::sim::run::ClientFut;
use crate::sim::selection::StrategyGraphShape;

/// The strategy contract version the wire manifest declares (honba-strategy's
/// `STRATEGY_API_VERSION`; the fold here mirrors it so both sides of the
/// compile handshake read the same constant).
pub const STRATEGY_API_VERSION: &str = "1.0.0";

/// The canonical manifest shape honba-ya verifies
/// (honba-strategy `StrategyManifest`, JSON field names).
pub fn strategy_manifest_value(graph: &StrategyGraphShape) -> Value {
    let instruments: Vec<Value> = graph
        .universe_explicit
        .iter()
        .map(|(symbol, exchange)| json!({ "symbol": symbol, "exchange": exchange }))
        .collect();
    // fe's `toManifest` passed the graph's universe through verbatim; the
    // named/explicit split of the wire contract maps the same two cases.
    let universe = if !graph.universe_named.is_empty() {
        json!({ "named": graph.universe_named })
    } else {
        json!({ "explicit": instruments })
    };
    json!({
        "api_version": STRATEGY_API_VERSION,
        "name": graph.name,
        "source_hash": source_hash(graph),
        "universe": universe,
        "subscriptions": {
            "instruments": instruments,
            "quotes": false,
            "trades": false,
        },
        "driving_timeframe": {
            "interval": graph.driving_interval,
            "aggregation": graph.driving_aggregation,
        },
        "warmup_bars": graph.warmup_bars,
        // fe's `toManifest` passes `nodes`/`edges` through verbatim (algodesigner
        // `manifest.ts`); the folded graph now carries them so the backend sees
        // the same manifest shape both ports write.
        "nodes": serde_json::to_value(&graph.nodes).unwrap_or_default(),
        "edges": serde_json::to_value(&graph.edges).unwrap_or_default(),
    })
}

/// The content-relevant digest standing in for `source_hash` (documented
/// deviation: FNV-1a-64 over the canonical graph JSON, not cryptographic —
/// the studio port owns real source digests).
pub fn source_hash(graph: &StrategyGraphShape) -> String {
    let Ok(canonical) = serde_json::to_string(graph) else {
        return String::new();
    };
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in canonical.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

/// What a compile attempt reports (fe `CompileResult`): the strategy id the
/// run names, and the compile error when it did not verify.
#[derive(Clone, Debug, PartialEq)]
pub struct CompileResult {
    /// The strategy id: the server's content id on success, the graph's
    /// display name otherwise (fe fell back so an uncompiled graph still
    /// names the template strategies).
    pub strategy_id: String,
    /// The compile failure, verbatim.
    pub error: Option<String>,
}

/// The client slice the compile step needs (fe `StrategyCompiler`).
pub trait StrategyCompiler {
    /// `POST /strategies` — verify + register the manifest.
    fn post_strategies_manifest<'a>(
        &'a self,
        manifest: &'a Value,
    ) -> ClientFut<'a, Result<CompiledStrategy, ClientError>>;
}

/// Verify + register the graph by compiling its manifest
/// (`compileStrategy`).
///
/// Unlike fe (with its `Partial<ApiClient>` structural checks), the seam is a
/// trait here, so "no route" cannot exist — every failure reaches the caller
/// as `CompileResult.error` and the store folds it into a failed run.
pub async fn compile_strategy<C: StrategyCompiler>(
    client: &C,
    graph: &StrategyGraphShape,
) -> CompileResult {
    let manifest = strategy_manifest_value(graph);
    match client.post_strategies_manifest(&manifest).await {
        Ok(compiled) => CompileResult {
            strategy_id: compiled.id,
            error: None,
        },
        Err(error) => CompileResult {
            strategy_id: graph.name.clone(),
            error: Some(error.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template() -> StrategyGraphShape {
        crate::sim::selection::builtin_template().0
    }

    struct OkCompile(CompiledStrategy);
    struct FailingCompile(String);

    impl StrategyCompiler for OkCompile {
        fn post_strategies_manifest<'a>(
            &'a self,
            _manifest: &'a Value,
        ) -> ClientFut<'a, Result<CompiledStrategy, ClientError>> {
            let strategy = self.0.clone();
            Box::pin(async move { Ok(strategy) })
        }
    }

    impl StrategyCompiler for FailingCompile {
        fn post_strategies_manifest<'a>(
            &'a self,
            _manifest: &'a Value,
        ) -> ClientFut<'a, Result<CompiledStrategy, ClientError>> {
            let message = self.0.clone();
            Box::pin(async move { Err(ClientError::Decode(message)) })
        }
    }

    #[test]
    fn the_manifest_carries_every_wire_required_field() {
        let manifest = strategy_manifest_value(&template());
        assert_eq!(manifest["api_version"], STRATEGY_API_VERSION);
        assert_eq!(manifest["name"], "SMA Cross");
        assert!(manifest["source_hash"].as_str().is_some_and(|h| h.starts_with("fnv1a64:")));
        assert_eq!(manifest["driving_timeframe"], json!({"interval": 5, "aggregation": "minute"}));
        assert_eq!(manifest["warmup_bars"], 2);
        assert_eq!(
            manifest["universe"],
            json!({ "explicit": [ { "symbol": "TCS", "exchange": "NSE" } ] })
        );
        assert_eq!(
            manifest["subscriptions"],
            json!({ "instruments": [ { "symbol": "TCS", "exchange": "NSE" } ], "quotes": false, "trades": false })
        );
        // A named universe maps to `{"named": …}` and wins over the explicit
        // list, exactly like fe's `universeString` precedence.
        let named = StrategyGraphShape {
            universe_named: "nifty50".to_string(),
            ..template()
        };
        assert_eq!(strategy_manifest_value(&named)["universe"], json!({ "named": "nifty50" }));
    }

    #[test]
    fn the_source_hash_separates_named_graphs() {
        assert_eq!(source_hash(&template()), source_hash(&template()));
        let other = StrategyGraphShape {
            driving_interval: 15,
            ..template()
        };
        assert_ne!(source_hash(&template()), source_hash(&other));
    }

    #[test]
    fn a_successful_compile_reports_the_content_id() {
        // The fixture way the server's `GET /strategies` answers (P0 wire
        // fixture), parsed exactly as the client would — the fake's payload
        // is a real `CompiledStrategy`, not a struct the test cheats into
        // shape.
        let value = serde_json::json!({
            "id": "sha256:da444fa65e98325f2f440ca450eeddf0aa93b88a412589e5b0d6f3666c790fd0",
            "ir": {
                "schema_version": 4,
                "strategy_api_version": "1.0.0",
                "manifest": {
                    "api_version": "1.0.0",
                    "name": "sma",
                    "source_hash": "sha256:abc",
                    "universe": { "explicit": [ { "symbol": "TCS", "exchange": "NSE" } ] },
                    "subscriptions": {
                        "instruments": [ { "symbol": "TCS", "exchange": "NSE" } ],
                        "quotes": false,
                        "trades": false
                    },
                    "driving_timeframe": { "interval": 5, "aggregation": "minute" },
                    "warmup_bars": 20
                },
                "universe": {
                    "named": null,
                    "instruments": [ { "symbol": "TCS", "exchange": "NSE" } ]
                },
                "subscriptions": {
                    "bars": [ { "symbol": "TCS", "exchange": "NSE" } ],
                    "quotes": [],
                    "trades": []
                },
                "driving_timeframe": { "interval": 5, "aggregation": "minute" },
                "timeframes": [ { "interval": 5, "aggregation": "minute" } ],
                "warmup_bars": 20
            }
        });
        let compiled: CompiledStrategy = serde_json::from_value(value).expect("fixture decodes");
        let client = OkCompile(compiled);
        let result =
            crate::sim::testing::block_on(async { compile_strategy(&client, &template()).await });
        assert_eq!(
            result.strategy_id,
            "sha256:da444fa65e98325f2f440ca450eeddf0aa93b88a412589e5b0d6f3666c790fd0"
        );
        assert_eq!(result.error, None);
    }

    #[test]
    fn a_failed_compile_carries_the_error_and_the_name_fallback() {
        let client = FailingCompile("manifest: bad shape".to_string());
        let result = crate::sim::testing::block_on(async { compile_strategy(&client, &template()).await });
        assert_eq!(result.error.as_deref(), Some("decode: manifest: bad shape"));
        assert_eq!(result.strategy_id, "SMA Cross");
    }
}
