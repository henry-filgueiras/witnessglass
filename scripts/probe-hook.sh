#!/usr/bin/env bash
#
# Dump one raw Claude Code hook payload, verbatim, and exit.
#
# This is a diagnostic, not part of WitnessGlass. It exists to answer questions
# of the form "did the integration send this field, or did our adapter drop it?"
# — and it can only answer them by being independent of the adapter. It shares
# no code with `witnessglass claude-hook`, parses nothing, validates nothing,
# and interprets nothing. Whatever arrives on stdin is appended as one line.
#
# It is deliberately not wired into arm.sh and not part of the example hook
# configuration. Install it with scripts/probe.sh, read the result, remove it.
#
# Usage, as a Claude Code command hook:
#   scripts/probe-hook.sh <output-file>
#
# Output goes under .witnessglass/, which is gitignored, because a raw hook
# payload contains everything a recording contains: commands, file contents,
# tool output, and whatever passed through them. It is not redacted and is not
# safe to share.

# No `set -e`. A probe that kills the session it is observing is a bad probe:
# every failure path below ends in exit 0, and a lost payload is a better
# outcome than a disrupted session.

OUT="${1:-}"
if [ -z "$OUT" ]; then
    # Nothing to stdout, ever — Claude reads a hook's stdout as a decision.
    echo "probe-hook: no output file given" >&2
    exit 0
fi

mkdir -p "$(dirname "$OUT")" 2>/dev/null

# One payload per line, appended. `cat` rather than any interpretation: the
# whole point is that these bytes have not been through a parser.
if ! { cat; printf '\n'; } >> "$OUT" 2>/dev/null; then
    echo "probe-hook: could not append to $OUT" >&2
fi

exit 0
