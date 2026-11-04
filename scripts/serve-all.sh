#!/usr/bin/env bash
# serve-all.sh — run the Honba web stack as a single, Ctrl+C-managed job.
#
# What it starts:
#   1. honba serve — REST/WebSocket API backend on 127.0.0.1:8000
#      (the [[proxy]] backend declared in Trunk.toml).
#   2. trunk serve — honba-kao web bundle on 127.0.0.1:8080, proxying
#      /api/ -> http://127.0.0.1:8000/ (rewritten to upstream paths).
#
# Tear-down rules:
#   * Ctrl+C (SIGINT) / SIGTERM kills BOTH processes.
#   * If either child exits, the sibling is stopped too, mirroring the exit.
#
# Preflight: refuses to start if port 8000 or 8080 is already bound, and
# prints the owning PID(s) so you can `kill` them or free the ports first.
#
# Env overrides:
#   BACKEND_ADDR   host:port for honba serve   (default 127.0.0.1:8000)
#   DATA_DIR       market-data directory       (default <workspace>/honba-data/data)
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KAO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
WS_ROOT="$(cd "${KAO_DIR}/.." && pwd)"

BACKEND_ADDR="${BACKEND_ADDR:-127.0.0.1:8000}"
DATA_DIR="${DATA_DIR:-${WS_ROOT}/honba-data/data}"
BACKEND_PORT="${BACKEND_ADDR##*:}"
FRONTEND_PORT=8080

BACKEND_PID=""
TRUNK_PID=""

port_owner() {
  ss -ltnp 2>/dev/null | grep "127.0.0.1:${1} " | grep -o 'pid=[0-9]*' | head -1 || true
}

# Exit non-zero (before starting anything) when either port is taken.
preflight() {
  local taken=0 name port owner
  for entry in "backend:${BACKEND_PORT}" "frontend:${FRONTEND_PORT}"; do
    name="${entry%%:*}"; port="${entry##*:}"
    owner="$(port_owner "${port}")"
    if [[ -n "${owner}" ]]; then
      printf 'serve-all: %s port %d is already in use (%s)\n' \
        "${name}" "${port}" "${owner}" >&2
      printf 'serve-all: stop it first, e.g. kill %s\n' "${owner##pid=}" >&2
      taken=1
    fi
  done
  return "${taken}"
}

cleanup() {
  trap '' INT TERM EXIT
  [[ -n "${TRUNK_PID}"   ]] && kill "${TRUNK_PID}"   2>/dev/null
  [[ -n "${BACKEND_PID}" ]] && kill "${BACKEND_PID}" 2>/dev/null
  wait 2>/dev/null
  echo "serve-all: both processes stopped"
}
trap cleanup INT TERM EXIT

preflight || exit 1

# 1. Backend (REST/WS API) — graceful SIGINT/SIGTERM shutdown is built in.
honba serve --data-dir "${DATA_DIR}" --addr "${BACKEND_ADDR}" &
BACKEND_PID=$!

# Wait up to ~15s for the backend to bind before bringing up Trunk.
for _ in $(seq 1 30); do
  if (exec 3<>"/dev/tcp/127.0.0.1/${BACKEND_PORT}") 2>/dev/null; then
    exec 3>&- 3<&- 2>/dev/null
    echo "serve-all: backend up on ${BACKEND_ADDR}"
    break
  fi
  sleep 0.5
done

# 2. Trunk web server (frontend + /api proxy). exec so the PID maps to trunk.
(cd "${KAO_DIR}" && exec trunk serve --config Trunk.toml --release) &
TRUNK_PID=$!
echo "serve-all: frontend on http://127.0.0.1:${FRONTEND_PORT} (Ctrl+C stops backend + frontend)"

# First child to die takes the stack down with it.
wait -n
STATUS=$?
echo "serve-all: a child exited (status=${STATUS}); stopping the sibling"
exit "${STATUS}"
