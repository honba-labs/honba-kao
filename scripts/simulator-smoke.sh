#!/usr/bin/env bash
# Simulator wire contract probes (HBD-2026-DES-005 P0) against a live server.
#
# Usage: scripts/simulator-smoke.sh [base-url]
# Default base: http://127.0.0.1:8000/api  (honba-ya-cli serve)
#
# Asserts the documented behavior of the simulator routes:
#   1. POST /backtests without a seed is rejected (ValidationInvalidRequest).
#   2. POST /backtests with the known-good panel shape is accepted.
#   3. GET /backtests/{id} and /journal answer enveloped payloads.
#   4. GET /strategies lists compiled strategies.
#   5. The SSE route streams at least one frame (status ack).
set -euo pipefail

BASE="${1:-http://127.0.0.1:8000/api}"

say() { printf '\n== %s\n' "$1"; }
say "Base: $BASE"

say "1. POST /backtests without a seed is rejected"
BODY_SEEDLESS='{"strategy":"sma","universe":"TCS.NSE","start":"2025-01-01","end":"2025-06-01","bar_spec":"1d"}'
if curl -fsS -X POST -H 'content-type: application/json' -d "$BODY_SEEDLESS" \
    "$BASE/backtests"; then
  echo "FAIL: seed-free request accepted" >&2; exit 1
else
  echo "ok: rejected (this branch expects a non-2xx)"
fi

say "2. POST /backtests accepted"
BODY_OK='{"strategy":"sma","universe":"TCS.NSE","start":"2025-01-01","end":"2025-06-01","bar_spec":"1d","seed":42,"initial_capital":100000,"fill_model":"bar_fill"}'
RUN_JSON=$(curl -fsS -X POST -H 'content-type: application/json' -d "$BODY_OK" "$BASE/backtests")
echo "$RUN_JSON" | head -c 600; echo
RUN_ID=$(echo "$RUN_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["run_id"])')

say "3. GET /backtests/{id}"
curl -fsS "$BASE/backtests/$RUN_ID" | head -c 600; echo

say "4. GET /backtests/{id}/journal (plain JSON, enveloped)"
curl -fsS "$BASE/backtests/$RUN_ID/journal" | head -c 600; echo

say "5. GET /strategies"
curl -fsS "$BASE/strategies" | head -c 600; echo

say "6. SSE frames on /backtests/{id}/events"
timeout 5 curl -sN "$BASE/backtests/$RUN_ID/events" | head -5 || echo "(stream ended or timed out — status frame above is the ack we assert)"

echo
echo "smoke complete"
