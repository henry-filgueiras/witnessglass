#!/usr/bin/env bash
#
# Check whether a WitnessGlass recording is complete, truncated, or corrupt,
# without displaying what it contains.
#
# A recording holds prompts, commands, absolute paths, file contents, and tool
# output. `witnessglass replay` answers the structural question but prints every
# record to do it, so asking "did the recorder survive?" has meant reading the
# flight. This asks the same question and throws the answer's body away.
#
# It adds no analysis of its own. `replay` is the sole parser and validator; its
# NDJSON stdout is discarded and its exit status is passed through, so there is
# exactly one implementation of what a recording says.
#
# Usage:
#   scripts/check-recording.sh <RECORDING>
#
# Exit codes, which are replay's own:
#   0  complete, valid recording
#   2  valid prefix, truncated tail — the recording stops mid-record
#   1  corruption, an unreadable or missing recording, an invalid invocation, a
#      missing binary, or replay reaching no verdict at all
#
# Exit 1 therefore means "this did not check out", not specifically "the
# recording is corrupt". Read the stderr line for which it was.
#
# Payload-silent, with one measured exception. Event bodies never reach stdout,
# which is discarded whole. stderr carries replay's own summary or error — line
# numbers, byte offsets, schema versions, sequence numbers, session ids — none
# of which are event payloads. The exception is a *corrupt* record: the parser's
# diagnostic can quote the bytes it choked on, and those bytes may be part of a
# payload. A recording that checks as corrupt is the one not to check on a
# shared terminal. See docs/claude-adapter.md.
#
# A file named literally `-h` or `--help` is read as the option, not as a path.
# Pass `./-h` for such a file.
#
# The recording is only ever read. Nothing here arms, disarms, builds, or looks
# inside .witnessglass/recordings.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$ROOT/target/debug/witnessglass"

usage() {
    sed -n '2,39p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

# Arity is checked before the help option, so no malformed invocation can reach
# exit 0 — the "recording is complete" code — by carrying a `-h` somewhere in it.
if [ "$#" -ne 1 ]; then
    echo "check-recording: exactly one recording path is required" >&2
    echo "usage: scripts/check-recording.sh <RECORDING>" >&2
    exit 1
fi

case "$1" in
    -h|--help) usage; exit 0 ;;
esac

RECORDING="$1"

# Checked here rather than left to a "command not found", because a missing
# build is an operator problem with a fix, not a statement about the recording.
[ -x "$BINARY" ] || {
    echo "check-recording: $BINARY is missing or not executable; run 'cargo build'" >&2
    exit 1
}

# Whether the recording exists, is readable, and parses is replay's judgement to
# make, not this script's. Every failure mode below arrives as its exit status.
# No pipeline is used, so `pipefail` cannot rewrite the verdict on the way back.
STATUS=0
"$BINARY" replay --recording "$RECORDING" >/dev/null || STATUS=$?

# replay exits 0, 1, or 2 and nothing else. Anything else means it never reached
# a verdict — killed by a signal, say — which is neither evidence that the
# recording is fine nor evidence that it is truncated. Reported as failure so
# this script's contract stays the three values documented above, and said out
# loud on stderr so a human is not left reading it as corruption.
case "$STATUS" in
    0|1|2) ;;
    *)
        echo "check-recording: replay exited $STATUS without reaching a verdict" >&2
        STATUS=1
        ;;
esac

exit "$STATUS"
