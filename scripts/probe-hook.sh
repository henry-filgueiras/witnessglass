#!/usr/bin/env bash
#
# Capture one raw Claude Code hook payload, verbatim, as one completed file.
#
# This is a diagnostic, not part of WitnessGlass. It exists to answer questions
# of the form "did the integration send this field, or did our adapter drop it?"
# — and it can only answer them by being independent of the adapter. It shares
# no code with `witnessglass claude-hook`, parses nothing, validates nothing,
# and interprets nothing. Whatever arrives on stdin becomes one file.
#
# Independent failure modes are not absent ones. This script can lose a payload,
# be installed on the wrong hooks, or be killed mid-capture. What it cannot do is
# fail in the same way the adapter fails, which is the only property the probe
# was ever built to have.
#
# It is deliberately not wired into arm.sh and not part of the example hook
# configuration. Install it with scripts/probe.sh, read the result, remove it.
#
# Usage, as a Claude Code command hook:
#   scripts/probe-hook.sh <spool-directory>
#
# One file per invocation, not one line appended to a shared file. Claude runs
# matching hooks in parallel, so several copies of this script can be writing at
# the same moment; a payload larger than a pipe buffer needs several writes, and
# two interleaved `cat >>` streams produce lines that belong to neither payload.
# A capture that cannot be trusted under concurrency cannot calibrate anything.
#
# Completion is a rename. The payload is written under <spool>/incomplete/ and
# moved into <spool>/ only once stdin has been consumed without error, so a file
# in the spool is a whole payload and a file left in incomplete/ is a partial
# one. That distinction is visible to a reader rather than inferred.
#
# Atomicity here is against interleaving and partial visibility, not against
# power loss: nothing is fsynced, so a machine that dies mid-capture may leave a
# renamed file whose bytes never reached the disk.
#
# Output goes under .witnessglass/, which is gitignored, because a raw hook
# payload contains everything a recording contains: commands, file contents,
# tool output, and whatever passed through them. It is not redacted and is not
# safe to share.

# No `set -e`. A probe that kills the session it is observing is a bad probe:
# every failure path below ends in exit 0, and a lost payload is a better
# outcome than a disrupted session.

SPOOL="${1:-}"
if [ -z "$SPOOL" ]; then
    # Nothing to stdout, ever — Claude reads a hook's stdout as a decision.
    echo "probe-hook: no spool directory given" >&2
    exit 0
fi

# An installation from before the spool existed passes the old single-file
# capture path. Never append to it: that is the shared-file failure this script
# was changed to remove, and the file may hold evidence from an earlier pass.
# Spool alongside it instead, and leave it exactly where it is.
if [ ! -d "$SPOOL" ] && { [ -e "$SPOOL" ] || [ "${SPOOL%.ndjson}" != "$SPOOL" ]; }; then
    SPOOL="$(dirname "$SPOOL")/payloads"
fi

INCOMPLETE="$SPOOL/incomplete"
mkdir -p "$INCOMPLETE" 2>/dev/null || {
    echo "probe-hook: could not create $INCOMPLETE" >&2
    exit 0
}

STAMP="$(date -u +%Y%m%dT%H%M%SZ 2>/dev/null)" || STAMP="undated"
TMP="$(mktemp "$INCOMPLETE/$STAMP-$$-XXXXXX" 2>/dev/null)" || {
    echo "probe-hook: could not create a capture file in $INCOMPLETE" >&2
    exit 0
}

# Raw bytes, in one file, unmodified. `cat` rather than any interpretation: the
# whole point is that these bytes have not been through a parser. Nothing is
# appended to them either — the previous version added a newline when the
# payload did not end in one, which is a modification, however small, to the
# evidence a reader is being asked to trust.
if ! cat >"$TMP" 2>/dev/null; then
    echo "probe-hook: capture failed; partial payload left at $TMP" >&2
    exit 0
fi

BASE="$(basename "$TMP")"
DEST="$SPOOL/$BASE.payload"
# `mktemp` already made the name unique inside incomplete/. This loop covers the
# remaining case — a name that somehow survives in the spool from an earlier run
# — because overwriting a captured payload is worse than an ugly file name.
n=0
while [ -e "$DEST" ]; do
    n=$((n + 1))
    DEST="$SPOOL/$BASE-$n.payload"
done

if ! mv "$TMP" "$DEST" 2>/dev/null; then
    echo "probe-hook: captured but could not complete $TMP" >&2
fi

exit 0
