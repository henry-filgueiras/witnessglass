#!/usr/bin/env bash
#
# Disarm WitnessGlass recording for this repository.
#
# Removes the hook configuration that scripts/arm.sh installed and restores any
# settings file that arming displaced. Safe to run when not armed.
#
# One rule governs everything here: this script never deletes a file it did not
# write byte-for-byte. If the settings file changed since arming, it is moved
# aside rather than removed, so an edit you made is never destroyed by a
# disarm.
#
# Recordings are left alone. Disarming stops recording; it does not discard
# evidence already captured.
#
# Usage:
#   scripts/disarm.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SETTINGS="$ROOT/.claude/settings.local.json"
EXAMPLE="$ROOT/.claude/settings.witnessglass.example.json"
STATE_DIR="$ROOT/.witnessglass"
SENTINEL="$STATE_DIR/armed"
RECORDINGS="$STATE_DIR/recordings"

for arg in "$@"; do
    case "$arg" in
        -h|--help) sed -n '2,16p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "disarm: unknown argument '$arg'" >&2; exit 1 ;;
    esac
done

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

# Read one key out of the sentinel. Empty when absent, which is a meaningful
# value here: an empty `displaced_settings` means arming displaced nothing.
sentinel_value() {
    [ -f "$SENTINEL" ] || return 0
    sed -n "s/^$1=//p" "$SENTINEL" | head -n 1
}

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"

if [ ! -f "$SENTINEL" ] && [ ! -f "$SETTINGS" ]; then
    echo "==> not armed; nothing to do"
    exit 0
fi

# ---------------------------------------------------------------------------
# Remove the hook configuration.
# ---------------------------------------------------------------------------
if [ -f "$SETTINGS" ]; then
    EXPECTED="$(sentinel_value settings_sha256)"
    ACTUAL="$(sha256 "$SETTINGS")"

    # The example is checked as well as the sentinel, so a configuration that is
    # byte-for-byte the one we install is recognised as ours even when the
    # sentinel is missing. That keeps a lost sentinel from downgrading a clean
    # removal into a pile of moved-aside copies.
    FROM_EXAMPLE=""
    if [ -f "$EXAMPLE" ] && [ "$ACTUAL" = "$(sha256 "$EXAMPLE")" ]; then
        FROM_EXAMPLE=1
    fi

    if { [ -n "$EXPECTED" ] && [ "$EXPECTED" = "$ACTUAL" ]; } || [ -n "$FROM_EXAMPLE" ]; then
        # Byte-for-byte what arming wrote. Ours to remove.
        rm -f "$SETTINGS"
        echo "==> removed $SETTINGS"
    elif grep -q 'claude-hook' "$SETTINGS"; then
        # Recognisably ours, but edited since arming, or armed without a
        # sentinel. Moved aside rather than deleted: the edit may have been
        # deliberate and it is not this script's to throw away.
        ASIDE="$SETTINGS.disarmed.$STAMP"
        mv "$SETTINGS" "$ASIDE"
        echo "==> settings.local.json changed since arming, so it was not deleted"
        echo "    moved to $ASIDE"
    else
        # Not ours at all. Left exactly where it is.
        echo "==> $SETTINGS is not a WitnessGlass configuration; left untouched"
        echo "    remove it yourself if you did not expect it to be there"
    fi
fi

# ---------------------------------------------------------------------------
# Restore whatever arming displaced.
# ---------------------------------------------------------------------------
DISPLACED="$(sentinel_value displaced_settings)"
if [ -n "$DISPLACED" ]; then
    if [ -f "$DISPLACED" ]; then
        if [ -e "$SETTINGS" ]; then
            echo "==> not restoring your previous settings: $SETTINGS still exists"
            echo "    yours is still at $DISPLACED"
        else
            mv "$DISPLACED" "$SETTINGS"
            echo "==> restored the settings.local.json that arming displaced"
        fi
    else
        echo "==> arming displaced $DISPLACED, but it is no longer there"
    fi
fi

rm -f "$SENTINEL"

echo "==> disarmed"

# ---------------------------------------------------------------------------
# Report what was captured. Recordings survive a disarm.
# ---------------------------------------------------------------------------
if [ -d "$RECORDINGS" ]; then
    COUNT="$(find "$RECORDINGS" -maxdepth 1 -name '*.ndjson' | wc -l | tr -d ' ')"
    if [ "$COUNT" != "0" ]; then
        echo
        echo "$COUNT recording(s) kept in $RECORDINGS"
        echo "  replay one: witnessglass replay --recording $RECORDINGS/<session-id>.ndjson"
        echo "  they are NOT redacted and are NOT safe to share"
    fi
fi

echo
echo "Takes effect for the NEXT Claude session. Whether an already-running"
echo "session drops its hooks when the configuration disappears is not something"
echo "this project has measured, so do not rely on either answer."
