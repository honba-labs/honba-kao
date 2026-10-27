#!/usr/bin/env bash
# honba-kao performance verification (Phase 8.4).
#
# Targets:
#   * Sub-100ms Time-to-Interactive (TTI)
#   * 60fps tick ingestion without frame reflow
#   * WASM binary size optimization via `wasm-opt` (binaryen)
#
# `wasm-opt` is not installed in this environment, so this script reports the
# targets, checks whether `wasm-opt` is available, and (once `trunk build`
# produces dist/) reports the WASM artifact size — those checks are
# informational. It also runs the automated tick-ingestion throughput/latency
# gate (tests/realtime_performance.rs); only that gate failing exits non-zero.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="$ROOT/dist"

echo "⚡ honba-kao performance verification (Phase 8.4)"
echo "  Targets: TTI <100ms | 60fps tick ingestion (no frame reflow) | WASM optimized via wasm-opt"
echo ""

# --- 1. wasm-opt availability ----------------------------------------------
echo "  Checking WASM optimizer (wasm-opt / binaryen)..."
if command -v wasm-opt >/dev/null 2>&1; then
  echo "    ✅ wasm-opt available: $(wasm-opt --version 2>/dev/null | head -1)"
  WASM_OPT=1
else
  echo "    ⚠️  wasm-opt NOT found — install binaryen (e.g. npm i -g wasm-opt or brew install binaryen)"
  echo "       then run: wasm-opt -O4 -o dist/honba_kao_app_bg.wasm dist/honba_kao_app_bg.wasm"
  WASM_OPT=0
fi

# --- 2. WASM artifact size (only meaningful after a successful trunk build) --
echo "  Checking WASM artifact size in dist/..."
WASM_FILES=$(find "$DIST" -maxdepth 1 -name '*.wasm' -type f 2>/dev/null || true)
if [ -z "$WASM_FILES" ]; then
  echo "    ℹ️  No WASM artifacts in dist/ yet (run trunk build first — honba-kao-app"
  echo "       currently has pre-existing compile errors blocking the build)."
else
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    size=$(stat -c %s "$f" 2>/dev/null || stat -f %z "$f" 2>/dev/null || echo 0)
    size_mb=$(awk -v b="$size" 'BEGIN{printf "%.2f", b/1048576}')
    echo "    📦 $(basename "$f"): ${size_mb} MiB"
    if [ "$size" -gt 5242880 ]; then
      echo "       ⚠️  exceeds 5 MiB uncompressed — run wasm-opt -O4 and enable gzip/brotli"
    else
      echo "       ✅ within 5 MiB uncompressed target"
    fi
  done <<< "$WASM_FILES"
fi

# --- 3. Tick-ingestion throughput / latency (automated, DoD #5) -------------
# Runs the native regression gate in crates/honba-kao-app/tests/
# realtime_performance.rs (parse_frame + QuoteBook::apply_frame over a burst
# of multi-symbol quote frames). A failure here is a real regression and makes
# this script exit non-zero. Detailed numbers: cargo bench -p honba-kao-app
# --bench realtime_feed
echo ""
echo "  Checking tick-ingestion throughput & latency (DoD #5: 100+ ticks/sec)..."
INGEST_OK=1
if INGEST_OUT=$(cd "$ROOT" && cargo test --release -p honba-kao-app --test realtime_performance -- --nocapture 2>&1); then
  echo "    ✅ ingestion gate passed"
else
  echo "    ❌ ingestion gate FAILED"
  INGEST_OK=0
fi
echo "$INGEST_OUT" | grep -E '^realtime_performance:|panicked|ticks/sec below|latency .* exceeds|^error'| sed 's/^/       /' || true

# --- 4. TTI / 60fps note (still manual: needs a real browser) ---------------
echo ""
echo "  TTI & 60fps rendering: verify with Chrome DevTools Performance / Lighthouse"
echo "  against http://127.0.0.1:8080 (trunk serve). Sub-100ms TTI and 60fps"
echo "  rendering require the market-feed WebSocket to render through ui-core"
echo "  primitives without full-page reflow (no React-style virtual DOM)."

echo ""
if [ "$INGEST_OK" -ne 1 ]; then
  echo "❌ Performance verification failed (tick-ingestion regression)"
  exit 1
fi
echo "✅ Performance verification complete"
exit 0
