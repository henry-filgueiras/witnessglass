#!/usr/bin/env bash
#
# Arm WitnessGlass recording for this repository.
#
# Copies the inert example hook configuration into .claude/settings.local.json,
# which Claude Code actually reads and which is gitignored. Recording then
# happens in the NEXT Claude session.
#
# Re-running this while already armed re-arms from scratch: it disarms first,
# rebuilds, and installs a fresh configuration, so "arm" always means "armed
# with the current build and the current example".
#
# The sentinel this writes is not a second copy of "am I armed" — the settings
# file is already that, and a duplicate flag would only drift from it. It is a
# record of what this script *did*, so that disarm.sh can undo exactly that and
# nothing else: whether it created the settings file or displaced one that was
# already there, and what the file looked like when it was written.
#
# Usage:
#   scripts/arm.sh              build, verify, then arm
#   scripts/arm.sh --no-build   skip cargo build (the binary must already exist)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

EXAMPLE="$ROOT/.claude/settings.witnessglass.example.json"
SETTINGS="$ROOT/.claude/settings.local.json"
STATE_DIR="$ROOT/.witnessglass"
SENTINEL="$STATE_DIR/armed"
RECORDINGS="$STATE_DIR/recordings"
BINARY="$ROOT/target/debug/witnessglass"

BUILD=1
for arg in "$@"; do
    case "$arg" in
        --no-build) BUILD=0 ;;
        -h|--help) sed -n '2,22p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "arm: unknown argument '$arg'" >&2; exit 1 ;;
    esac
done

# sha256 of a file, on both macOS and Linux.
sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

[ -f "$EXAMPLE" ] || { echo "arm: missing $EXAMPLE" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Re-arm cleanly if anything is already in place.
#
# The settings file is checked as well as the sentinel, so a deleted sentinel
# cannot strand an armed configuration that nothing knows how to remove.
# ---------------------------------------------------------------------------
if [ -f "$SENTINEL" ] || { [ -f "$SETTINGS" ] && grep -q 'claude-hook' "$SETTINGS"; }; then
    echo "==> already armed; disarming first"
    "$ROOT/scripts/disarm.sh"
    echo
fi

# ---------------------------------------------------------------------------
# Build. The hooks invoke the built binary directly rather than `cargo run`, so
# a stale binary would quietly record a real session using old code.
# ---------------------------------------------------------------------------
if [ "$BUILD" -eq 1 ]; then
    echo "==> cargo build"
    (cd "$ROOT" && cargo build)
fi

[ -x "$BINARY" ] || {
    echo "arm: $BINARY is missing or not executable; run 'cargo build'" >&2
    exit 1
}

# ---------------------------------------------------------------------------
# Verify the binary before trusting it with a session, using a synthetic payload
# in a throwaway directory. Better to find a broken adapter now than halfway
# through the session it was supposed to record.
# ---------------------------------------------------------------------------
echo "==> verifying the adapter against a synthetic payload"
PROBE="$(mktemp -d)"
trap 'rm -rf "$PROBE"' EXIT

PROBE_OUT="$(printf '%s' \
    '{"hook_event_name":"SessionStart","session_id":"arm-selftest","source":"startup"}' \
    | "$BINARY" claude-hook --recordings-dir "$PROBE/recordings")" || {
    echo "arm: the adapter failed its self-test; not arming" >&2
    exit 1
}
if [ -n "$PROBE_OUT" ]; then
    echo "arm: the adapter wrote to stdout, which Claude reads as a decision; not arming" >&2
    exit 1
fi
[ -f "$PROBE/recordings/arm-selftest.ndjson" ] || {
    echo "arm: the adapter wrote no recording during its self-test; not arming" >&2
    exit 1
}

# ---------------------------------------------------------------------------
# Displace any pre-existing settings rather than overwriting them. A settings
# file that reached this point is not ours — the re-arm branch above already
# removed one that was.
# ---------------------------------------------------------------------------
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
DISPLACED=""
if [ -f "$SETTINGS" ]; then
    DISPLACED="$SETTINGS.displaced-by-witnessglass.$STAMP"
    mv "$SETTINGS" "$DISPLACED"
    echo "==> moved your existing settings.local.json aside"
    echo "    $DISPLACED"
    echo "    disarm.sh will put it back."
fi

mkdir -p "$RECORDINGS"
cp "$EXAMPLE" "$SETTINGS"

cat > "$SENTINEL" <<EOF
# WitnessGlass arm sentinel. Written by scripts/arm.sh; removed by scripts/disarm.sh.
# This records what arming did, so disarming can undo exactly that. Editing it by
# hand will confuse disarm.sh; delete the settings file yourself instead.
armed_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
witnessglass_version=$("$BINARY" --version)
binary=$BINARY
binary_sha256=$(sha256 "$BINARY")
settings=$SETTINGS
settings_sha256=$(sha256 "$SETTINGS")
displaced_settings=$DISPLACED
recordings_dir=$RECORDINGS
EOF

echo "==> armed"
echo
echo "    hooks:      $SETTINGS"
echo "    recordings: $RECORDINGS/<session-id>.ndjson"
echo "    binary:     $BINARY ($("$BINARY" --version))"
echo
echo "Recording starts with the NEXT Claude session. This will not retroactively"
echo "record the session you are in now, and arming mid-session would produce a"
echo "partial recording with no session boundary."
echo
echo "Recordings are NOT redacted and are NOT safe to share."
echo "Disarm with: scripts/disarm.sh"
