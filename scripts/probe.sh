#!/usr/bin/env bash
#
# Install, remove, and read a raw hook-payload probe.
#
# The probe is an independent observer of what Claude Code actually sends. It
# does not go through the WitnessGlass adapter, so it can distinguish "the
# integration did not send this field" from "our adapter dropped it" — a
# distinction a field-population count cannot make, and one that dragon:1 turned
# on.
#
# It attaches to the three hooks that document an optional `duration`:
# PostToolUse, PostToolUseFailure, and PermissionDenied. Only the last of those
# has ever been observed carrying one, and only from a tool's own response body.
#
# Usage:
#   scripts/probe.sh install   add the probe hooks to .claude/settings.local.json
#   scripts/probe.sh remove    take them out again, leaving everything else alone
#   scripts/probe.sh show      summarise what was captured, WITHOUT printing payloads
#   scripts/probe.sh dump      print the raw capture (SENSITIVE — see below)
#   scripts/probe.sh clear     delete the capture file
#
# `install` works whether or not WitnessGlass is armed; it merges into an
# existing configuration rather than replacing one. Running it alongside an
# armed session is the useful case: the recording and the raw payloads then
# describe the same events, and can be compared.
#
# `show` is payload-quiet by design, in the same spirit as check-recording.sh:
# it reports which keys arrived, not what was in them, so the question can be
# answered on a shared screen. `dump` is not. A raw payload contains commands,
# file contents, tool output, and anything that passed through them. It is not
# redacted and is not safe to share.
#
# After arming, disarm.sh will notice settings.local.json no longer matches what
# arming wrote and will move it aside rather than delete it. That is correct
# behaviour and not an error.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SETTINGS="$ROOT/.claude/settings.local.json"
CAPTURE="$ROOT/.witnessglass/probe/raw-hooks.ndjson"
PROBE="$ROOT/scripts/probe-hook.sh"

# The hooks that document an optional `duration`.
HOOKS="PostToolUse PostToolUseFailure PermissionDenied"

usage() { sed -n '2,40p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; }

command -v python3 >/dev/null 2>&1 || {
    echo "probe: python3 is required to edit the settings JSON" >&2
    exit 1
}

case "${1:-}" in
install)
    [ -x "$PROBE" ] || { echo "probe: $PROBE is not executable" >&2; exit 1; }
    mkdir -p "$(dirname "$SETTINGS")" "$(dirname "$CAPTURE")"
    python3 - "$SETTINGS" "$CAPTURE" $HOOKS <<'PY'
import json, os, sys
settings_path, capture, *hooks = sys.argv[1:]
settings = {}
if os.path.exists(settings_path):
    with open(settings_path, encoding="utf-8") as f:
        settings = json.load(f)
entry = {
    "matcher": "*",
    "hooks": [{
        "type": "command",
        "command": "${CLAUDE_PROJECT_DIR}/scripts/probe-hook.sh",
        "args": ["${CLAUDE_PROJECT_DIR}/.witnessglass/probe/raw-hooks.ndjson"],
        "timeout": 10,
        "statusMessage": "probe: dumping raw hook payload",
    }],
}
def is_probe(block):
    return any("probe-hook.sh" in h.get("command", "") for h in block.get("hooks", []))
settings.setdefault("hooks", {})
added = []
for hook in hooks:
    blocks = settings["hooks"].setdefault(hook, [])
    if any(is_probe(b) for b in blocks):
        continue
    blocks.append(json.loads(json.dumps(entry)))
    added.append(hook)
with open(settings_path, "w", encoding="utf-8") as f:
    json.dump(settings, f, indent=2)
    f.write("\n")
print("added to: " + (", ".join(added) if added else "(already present)"))
PY
    echo "==> probe installed in $SETTINGS"
    echo "    captures to: $CAPTURE"
    echo
    echo "It takes effect in the NEXT Claude session. Raw payloads are as sensitive"
    echo "as a recording: not redacted, not safe to share."
    ;;

remove)
    [ -f "$SETTINGS" ] || { echo "probe: $SETTINGS does not exist; nothing to remove"; exit 0; }
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
    echo "    the capture file is left alone; scripts/probe.sh clear deletes it"
    ;;

show)
    [ -f "$CAPTURE" ] || { echo "probe: no capture at $CAPTURE"; exit 1; }
    python3 - "$CAPTURE" <<'PY'
import json, sys
from collections import Counter, defaultdict
keys = defaultdict(Counter)
counts = Counter()
duration = defaultdict(Counter)
tools = defaultdict(Counter)
unparsed = 0
for line in open(sys.argv[1], encoding="utf-8"):
    line = line.strip()
    if not line:
        continue
    try:
        payload = json.loads(line)
    except Exception:
        unparsed += 1
        continue
    hook = payload.get("hook_event_name", "(no hook_event_name)")
    counts[hook] += 1
    for k in payload:
        keys[hook][k] += 1
    tool = payload.get("tool_name")
    if tool:
        tools[hook][tool] += 1
    duration[hook]["present" if "duration" in payload else "absent"] += 1

print("Raw hook payloads captured, key names only. No values are printed.\n")
if unparsed:
    print(f"!! {unparsed} line(s) did not parse as JSON\n")
for hook in sorted(counts):
    print(f"{hook}  ({counts[hook]} payload(s))")
    d = duration[hook]
    verdict = "PRESENT on %d, absent on %d" % (d["present"], d["absent"])
    flag = "  <-- the field in question" if d["present"] else ""
    print(f"  top-level `duration`: {verdict}{flag}")
    if tools[hook]:
        print("  tools: " + ", ".join(f"{t}={n}" for t, n in sorted(tools[hook].items())))
    print("  top-level keys: " + ", ".join(sorted(keys[hook])))
    print()
print("A key named here arrived; a key absent here did not. What was *in* any of")
print("them is not shown, and `scripts/probe.sh dump` is the only thing that shows it.")
PY
    ;;

dump)
    [ -f "$CAPTURE" ] || { echo "probe: no capture at $CAPTURE"; exit 1; }
    echo "probe: this prints raw hook payloads — commands, file contents, and tool" >&2
    echo "       output, unredacted. Not safe on a shared screen or in an issue." >&2
    echo >&2
    cat "$CAPTURE"
    ;;

clear)
    rm -f "$CAPTURE"
    echo "==> removed $CAPTURE"
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
