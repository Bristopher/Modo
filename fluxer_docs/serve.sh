#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV="$HERE/.venv"
ADDR="${ZENSICAL_DEV_ADDR:-0.0.0.0:8000}"
LOG="${ZENSICAL_LOG:-/tmp/zensical-serve.log}"
PORT="${ADDR##*:}"

cd "$HERE"

ensure_env() {
	if [ ! -x "$VENV/bin/python" ]; then
		python3 -m venv "$VENV"
		"$VENV/bin/python" -m pip install --quiet --upgrade pip
	fi
	"$VENV/bin/python" -m pip install --quiet --require-virtualenv -r "$HERE/requirements.txt"
}

case "${1:-serve}" in
	--bootstrap)
		ensure_env
		;;
	--daemon)
		ensure_env
		if curl -sf -o /dev/null "http://127.0.0.1:${PORT}/" 2>/dev/null; then
			echo "zensical already serving on ${ADDR}"
			exit 0
		fi
		setsid "$VENV/bin/zensical" serve -a "$ADDR" >"$LOG" 2>&1 &
		disown 2>/dev/null || true
		echo "zensical serving on ${ADDR} (logs: ${LOG})"
		;;
	serve)
		ensure_env
		exec "$VENV/bin/zensical" serve -a "$ADDR"
		;;
	*)
		ensure_env
		exec "$VENV/bin/zensical" "$@"
		;;
esac
