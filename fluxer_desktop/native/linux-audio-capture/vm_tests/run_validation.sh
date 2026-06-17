#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later

set -uo pipefail

WORK="${FLX_WORK:-/home/parallels/flx-vmtest}"
ADDON_SO="${FLX_ADDON_SO:-/home/parallels/flx-target/debug/libfluxer_linux_audio_capture.so}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mkdir -p "$WORK"
export XDG_RUNTIME_DIR="$WORK/xdg"
mkdir -p "$XDG_RUNTIME_DIR"; chmod 700 "$XDG_RUNTIME_DIR"
export PIPEWIRE_RUNTIME_DIR="$XDG_RUNTIME_DIR"
export PULSE_RUNTIME_PATH="$XDG_RUNTIME_DIR/pulse"
unset DISPLAY WAYLAND_DISPLAY DBUS_SESSION_BUS_ADDRESS

PIDS=()
cleanup() {
  for p in "${PIDS[@]:-}"; do kill -9 "$p" 2>/dev/null; done
  pkill -9 -u "$(id -un)" -f "pipewire" 2>/dev/null
  pkill -9 -u "$(id -un)" -f "wireplumber" 2>/dev/null
}
trap cleanup EXIT

echo "=== starting private pipewire server (runtime=$XDG_RUNTIME_DIR) ==="
pipewire        >"$WORK/pipewire.log"       2>&1 & PIDS+=($!)
sleep 0.8
pipewire-pulse  >"$WORK/pipewire-pulse.log" 2>&1 & PIDS+=($!)
sleep 0.5
wireplumber     >"$WORK/wireplumber.log"    2>&1 & PIDS+=($!)

for _ in $(seq 1 40); do pw-cli info 0 >/dev/null 2>&1 && break; sleep 0.25; done
if ! pw-cli info 0 >/dev/null 2>&1; then
  echo "FATAL: private pipewire did not come up"; cat "$WORK/pipewire.log"; exit 2
fi

echo "=== creating virtual speakers (default sink) ==="
pactl load-module module-null-sink sink_name=test_speakers sink_properties='device.description=Test_Speakers' >/dev/null 2>&1
pactl set-default-sink test_speakers 2>/dev/null
sleep 0.4

echo "=== generating a real 600s stereo tone ==="
TONE="$WORK/tone.wav"
[ -f "$TONE" ] || ffmpeg -hide_banner -loglevel error -f lavfi -i "sine=frequency=440:duration=600" -ac 2 -ar 48000 "$TONE" </dev/null
export FLX_TONE="$TONE"

echo "=== spawning 4 external real-app players (outside the harness process tree) ==="
pw-play --target test_speakers "$TONE" >/dev/null 2>&1 & PIDS+=($!)
pw-play --target test_speakers -P '{ application.name = "Music Player Demo" }' "$TONE" >/dev/null 2>&1 & PIDS+=($!)
pw-play --target test_speakers -P '{ application.name = "Fluxer" }' "$TONE" >/dev/null 2>&1 & PIDS+=($!)
pw-play --target test_speakers -P '{ node.name = "Fluxer Helper Stream" }' "$TONE" >/dev/null 2>&1 & PIDS+=($!)
sleep 1.5

echo "=== pre-test graph (fluxer sink should be ABSENT) ==="
pw-dump | node -e 'const d=JSON.parse(require("fs").readFileSync(0));const f=d.filter(o=>o.type==="PipeWire:Interface:Node"&&/fluxer-screen-share/.test(o.info?.props?.["node.name"]||""));console.log("fluxer sink nodes pre-test:",f.length);'

echo "=== running napi SYSTEM-capture validation harness ==="
export FLX_ADDON_SO="$ADDON_SO" FLX_WORK="$WORK"
node "$HERE/pw_graph_validation.mjs"
HARNESS_RC=$?

echo "=== running napi DIRECT (per-process) capture validation harness ==="
node "$HERE/direct_capture_validation.mjs"
DIRECT_RC=$?
[ "$DIRECT_RC" = "0" ] || HARNESS_RC=$DIRECT_RC

echo "=== post-exit cleanup check (Drop must remove the fluxer sink) ==="
sleep 0.8
RESIDUAL=$(pw-dump | node -e 'const d=JSON.parse(require("fs").readFileSync(0));const f=d.filter(o=>(o.type==="PipeWire:Interface:Node"&&/fluxer/.test(o.info?.props?.["node.name"]||""))||(o.type==="PipeWire:Interface:Link"&&/fluxer/.test(JSON.stringify(o.info?.props||{}))));console.log(f.length);')
if [ "$RESIDUAL" = "0" ]; then
  echo "PASS  no fluxer nodes/links remain after addon process exit (clean teardown)"
else
  echo "FAIL  $RESIDUAL residual fluxer objects after addon process exit"
  HARNESS_RC=1
fi

echo "=== DONE rc=$HARNESS_RC ==="
exit $HARNESS_RC
