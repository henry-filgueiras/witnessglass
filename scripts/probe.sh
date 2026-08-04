#!/usr/bin/env bash
#
# Install, remove, and read a raw hook-payload probe.
#
# The probe is an independent observer of what Claude Code actually sends. It
# does not go through the WitnessGlass adapter, so its failure modes are
# independent of the adapter's — which is what lets it tell "the integration did
# not send this field" apart from "our adapter dropped it", a distinction a
# field-population count over a recording cannot make, and one that dragon:1
# turned on.
#
# Independent is not infallible. The probe can be installed on the wrong hooks,
# lose a payload if a hook process is killed, or be read wrongly. `show` in
# particular is a parser and a summary: it is the one part of this tool that has
# a model of the payload, and a payload it cannot parse is reported rather than
# counted. The raw evidence is the captured files, not this summary of them.
#
# By default it attaches to the three hooks that document an optional duration:
# PostToolUse, PostToolUseFailure, and PermissionDenied. Its first run found
# `duration_ms` populated on every completion of both hooks that fired, against
# an adapter that had been reading `duration` since it was written — which is
# the distinction above, doing exactly the job it was built for.
#
# Usage:
#   scripts/probe.sh install   add the probe hooks to .claude/settings.local.json
#   scripts/probe.sh remove    take them out again, leaving everything else alone
#   scripts/probe.sh show      summarise what was captured, WITHOUT printing payloads
#   scripts/probe.sh dump      print the raw captures (SENSITIVE — see below)
#   scripts/probe.sh clear     delete the captures this probe owns
#
# `install` works whether or not WitnessGlass is armed; it merges into an
# existing configuration rather than replacing one. Running it alongside an
# armed session is the useful case: the recording and the raw payloads then
# describe the same events, and can be compared. Set PROBE_HOOKS to a
# space-separated list to attach to different hooks than the default.
#
# `remove` takes out every probe block it finds, whichever hooks they are on,
# so an unusual PROBE_HOOKS does not need to be repeated to undo it.
#
# Each captured payload is one file under .witnessglass/probe/payloads, written
# elsewhere and renamed into place, because Claude runs matching hooks in
# parallel and concurrent appends to one file interleave. A file in that
# directory is a complete capture; a file under payloads/incomplete is a partial
# one, and `show` counts them separately rather than parsing them.
#
# `show` is payload-quiet by design, in the same spirit as check-recording.sh:
# hook names, top-level key names, counts, duration-shaped key names, and parse
# failures. No value from a payload is printed, including tool names — those are
# payload contents, and a summary that is safe on a shared screen cannot make an
# exception for the ones that look harmless. `dump` is not payload-quiet. A raw
# payload contains commands, file contents, tool output, and anything that
# passed through them. It is not redacted and is not safe to share.
#
# `clear` deletes only the captures under .witnessglass/probe/payloads and says
# how many. A pre-spool .witnessglass/probe/raw-hooks.ndjson from an earlier
# round is still read by `show` and `dump`, and is never deleted here: it is raw
# evidence behind findings already recorded, and removing it is a decision for a
# human with `rm`.
#
# After arming, disarm.sh will notice settings.local.json no longer matches what
# arming wrote and will move it aside rather than delete it. That is correct
# behaviour and not an error.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SETTINGS="$ROOT/.claude/settings.local.json"
PROBE_DIR="$ROOT/.witnessglass/probe"
SPOOL="$PROBE_DIR/payloads"
PROBE="$ROOT/scripts/probe-hook.sh"

# The hooks that document an optional duration. Override with PROBE_HOOKS to
# observe others — `PROBE_HOOKS="SubagentStart SubagentStop"` is the useful one,
# because dragon:1's parentage findings were established from adapter output and
# were only confirmed once a raw payload from those hooks was inspected. More
# hooks means more captured payloads, and a payload is as sensitive as a
# recording, so this is opt-in rather than the default.
HOOKS="${PROBE_HOOKS:-PostToolUse PostToolUseFailure PermissionDenied}"

# Print this file's own header comment, so usage cannot drift from it.
usage() {
    awk 'NR == 1 { next } /^#/ { sub(/^# ?/, ""); print; next } { exit }' "${BASH_SOURCE[0]}"
}

command -v python3 >/dev/null 2>&1 || {
    echo "probe: python3 is required to edit the settings JSON" >&2
    exit 1
}

case "${1:-}" in
install)
    [ -x "$PROBE" ] || {
        echo "probe: $PROBE is not executable" >&2
        exit 1
    }
    mkdir -p "$(dirname "$SETTINGS")" "$SPOOL"
    python3 - "$SETTINGS" $HOOKS <<'PY'
import json, os, sys
settings_path, *hooks = sys.argv[1:]
settings = {}
if os.path.exists(settings_path):
    with open(settings_path, encoding="utf-8") as f:
        settings = json.load(f)
entry = {
    "matcher": "*",
    "hooks": [{
        "type": "command",
        "command": "${CLAUDE_PROJECT_DIR}/scripts/probe-hook.sh",
        "args": ["${CLAUDE_PROJECT_DIR}/.witnessglass/probe/payloads"],
        "timeout": 10,
        "statusMessage": "probe: capturing raw hook payload",
    }],
}
def is_probe(block):
    return any("probe-hook.sh" in h.get("command", "") for h in block.get("hooks", []))
settings.setdefault("hooks", {})
added = []
for hook in hooks:
    blocks = settings["hooks"].setdefault(hook, [])
    # An installation from an earlier round points at the single-file capture.
    # Replace it rather than leaving two probe blocks on one hook: the capture
    # side spools either way, so two blocks would only capture everything twice.
    blocks[:] = [b for b in blocks if not is_probe(b)]
    blocks.append(json.loads(json.dumps(entry)))
    added.append(hook)
with open(settings_path, "w", encoding="utf-8") as f:
    json.dump(settings, f, indent=2)
    f.write("\n")
print("installed on: " + ", ".join(added))
PY
    echo "==> probe installed in $SETTINGS"
    echo "    captures to: $SPOOL (one file per hook invocation)"
    if [ -e "$PROBE_DIR/raw-hooks.ndjson" ]; then
        echo
        echo "    A pre-spool capture is still present at $PROBE_DIR/raw-hooks.ndjson."
        echo "    It is left untouched, is still read by 'show' and 'dump', and its"
        echo "    payloads will be reported alongside this session's. Move it aside"
        echo "    yourself if you want this session counted on its own."
    fi
    echo
    echo "It takes effect in the NEXT Claude session. Raw payloads are as sensitive"
    echo "as a recording: not redacted, not safe to share."
    ;;

remove)
    [ -f "$SETTINGS" ] || {
        echo "probe: $SETTINGS does not exist; nothing to remove"
        exit 0
    }
    python3 - "$SETTINGS" <<'PY'
import json, os, sys
path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    settings = json.load(f)
def is_probe(block):
    return any("probe-hook.sh" in h.get("command", "") for h in block.get("hooks", []))
removed = 0
for hook, blocks in list(settings.get("hooks", {}).items()):
    kept = [b for b in blocks if not is_probe(b)]
    removed += len(blocks) - len(kept)
    if kept:
        settings["hooks"][hook] = kept
    else:
        del settings["hooks"][hook]
if not settings.get("hooks"):
    settings.pop("hooks", None)

# A settings file left holding nothing is deleted rather than kept. An empty
# `{}` would look to arm.sh like a pre-existing configuration of yours, and it
# would dutifully move it aside on the next arm — a confusing artefact of a
# probe that is supposed to leave no trace.
if settings:
    with open(path, "w", encoding="utf-8") as f:
        json.dump(settings, f, indent=2)
        f.write("\n")
    print(f"removed {removed} probe hook block(s)")
else:
    os.remove(path)
    print(f"removed {removed} probe hook block(s); the file held nothing else, so it is gone")
PY
    echo "==> probe removed from $SETTINGS"
    echo "    captured payloads are left alone; scripts/probe.sh clear deletes them"
    ;;

show)
    python3 - "$SPOOL" "$PROBE_DIR" <<'PY'
import glob, json, os, sys
from collections import Counter, defaultdict

spool, probe_dir = sys.argv[1], sys.argv[2]
keys = defaultdict(Counter)
counts = Counter()
timing = defaultdict(lambda: defaultdict(Counter))
tool_values = defaultdict(set)
unparsed = []

def observe(payload, origin):
    hook = payload.get("hook_event_name", "(no hook_event_name)")
    counts[hook] += 1
    for k in payload:
        keys[hook][k] += 1
        # Match any duration-shaped key rather than one spelling. The first probe
        # run asked whether `duration` was present, printed "PRESENT on 0", and
        # listed `duration_ms` in the key line immediately below it. Naming the
        # field in advance is how the adapter missed it for two sprints; the probe
        # should not repeat that by asking a narrower question than it can answer.
        if "duration" in k.lower():
            timing[hook][k]["present" if payload[k] is not None else "null"] += 1
    tool = payload.get("tool_name")
    if tool is not None:
        # Counted, never named: a tool name is a payload value.
        tool_values[hook].add(repr(tool))

# One completed capture per file. A file that does not parse is named and
# counted, never quoted — the bytes it holds are exactly what nobody has
# established is safe to print.
captures = sorted(glob.glob(os.path.join(spool, "*.payload")))
for path in captures:
    try:
        with open(path, encoding="utf-8") as f:
            observe(json.load(f), path)
    except Exception as err:
        unparsed.append((os.path.basename(path), type(err).__name__))

# Pre-spool captures, if any survive. Read, never rewritten, never deleted.
legacy_files = sorted(
    p for p in glob.glob(os.path.join(probe_dir, "raw-hooks.ndjson*")) if os.path.isfile(p)
)
legacy_lines = 0
for path in legacy_files:
    with open(path, encoding="utf-8", errors="replace") as f:
        for number, line in enumerate(f, start=1):
            line = line.strip()
            if not line:
                continue
            legacy_lines += 1
            try:
                observe(json.loads(line), path)
            except Exception as err:
                unparsed.append((f"{os.path.basename(path)}:{number}", type(err).__name__))

incomplete = []
if os.path.isdir(os.path.join(spool, "incomplete")):
    incomplete = sorted(
        p for p in glob.glob(os.path.join(spool, "incomplete", "*")) if os.path.isfile(p)
    )

total = len(captures) + legacy_lines
if total == 0 and not incomplete:
    print(f"probe: no captures under {spool}")
    if not os.path.isdir(spool):
        print("       (the spool directory does not exist; the probe has not run here)")
    sys.exit(1)

print("Raw hook payloads captured, key names only. No values are printed.\n")
unparsed_captures = sum(1 for name, _ in unparsed if not name.startswith("raw-hooks.ndjson"))
parsed_note = f", {len(captures) - unparsed_captures} of them parsed" if unparsed_captures else ""
print(f"{len(captures)} completed capture file(s) in {spool}{parsed_note}")
for path in legacy_files:
    print(f"pre-spool capture still present: {path}")
if legacy_lines:
    print(f"{legacy_lines} payload(s) read from it, and counted below alongside the rest")
if incomplete:
    print(
        f"!! {len(incomplete)} incomplete capture(s) under {os.path.join(spool, 'incomplete')} — "
        "a hook process that was killed or could not finish writing. Not parsed, not counted."
    )
print()

if unparsed:
    print(f"!! {len(unparsed)} capture(s) did not parse as JSON:")
    for name, err in unparsed:
        print(f"     {name}  ({err})")
    print("   Bytes are not quoted here. `scripts/probe.sh dump` shows them, and is sensitive.")
    print()

for hook in sorted(counts):
    print(f"{hook}  ({counts[hook]} payload(s))")
    hook_total = counts[hook]
    if timing[hook]:
        for k in sorted(timing[hook]):
            c = timing[hook][k]
            print(
                f"  duration-shaped key `{k}`: value on {c['present']}, "
                f"null on {c['null']}, absent on {hook_total - c['present'] - c['null']}"
            )
    else:
        print(f"  duration-shaped keys: none on any of {hook_total} payload(s)")
    if tool_values[hook]:
        print(f"  distinct tool_name values: {len(tool_values[hook])} (not named; they are payload)")
    print("  top-level keys: " + ", ".join(sorted(keys[hook])))
    print()

print("A key named here arrived; a key absent here did not arrive on a payload this")
print("summary could parse. What was *in* any of them is not shown, and")
print("`scripts/probe.sh dump` is the only thing that shows it.")
PY
    ;;

dump)
    echo "probe: this prints raw hook payloads — commands, file contents, and tool" >&2
    echo "       output, unredacted. Not safe on a shared screen or in an issue." >&2
    echo >&2
    found=0
    for capture in "$SPOOL"/*.payload; do
        [ -e "$capture" ] || continue
        found=1
        echo "==== $capture"
        cat "$capture"
        echo
    done
    for legacy in "$PROBE_DIR"/raw-hooks.ndjson*; do
        [ -f "$legacy" ] || continue
        found=1
        echo "==== $legacy (pre-spool capture, one payload per line)"
        cat "$legacy"
    done
    for partial in "$SPOOL"/incomplete/*; do
        [ -f "$partial" ] || continue
        found=1
        echo "==== $partial (INCOMPLETE — a partial payload, not a whole one)"
        cat "$partial"
        echo
    done
    if [ "$found" -eq 0 ]; then
        echo "probe: no captures under $SPOOL" >&2
        exit 1
    fi
    ;;

clear)
    removed=0
    for capture in "$SPOOL"/*.payload; do
        [ -e "$capture" ] || continue
        rm -f "$capture"
        removed=$((removed + 1))
    done
    partials=0
    for partial in "$SPOOL"/incomplete/*; do
        [ -e "$partial" ] || continue
        rm -f "$partial"
        partials=$((partials + 1))
    done
    rmdir "$SPOOL/incomplete" "$SPOOL" 2>/dev/null || true
    echo "==> removed $removed completed capture(s) and $partials incomplete one(s) from $SPOOL"
    for legacy in "$PROBE_DIR"/raw-hooks.ndjson*; do
        [ -f "$legacy" ] || continue
        echo "    KEPT: $legacy — a pre-spool capture this command does not own."
        echo "          It is raw evidence behind findings already recorded. Delete it"
        echo "          yourself if you have decided it is finished with."
    done
    ;;

-h | --help | help | "")
    usage
    ;;

*)
    echo "probe: unknown command '${1}'" >&2
    echo >&2
    usage >&2
    exit 1
    ;;
esac
