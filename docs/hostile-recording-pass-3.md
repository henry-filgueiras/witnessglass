---
runme:
  version: v3
shell: bash
cwd: ..
---

# Hostile recording, pass 3

Pass 2 answered two of its five questions, disproved a finding this project had been carrying
for two sprints, and failed at both of the jobs that needed a human. This pass exists to
retry the two that failed, plus one that pass 2 made newly urgent.

It is executable. Every shell block below is a named [Runme](https://runme.dev) cell, and each
one writes its output to `.witnessglass/pass-3/`, so the transcript assembles itself instead of
being pasted together by hand.

```text {"name":"invocation","excludeFromRunAll":"true"}
runme ls  --filename docs/hostile-recording-pass-3.md          # the cells, in order
runme run --filename docs/hostile-recording-pass-3.md arm      # one cell, by name
```

**There is deliberately no `runme run --all`.** Every cell is marked `excludeFromRunAll`, because
a whole Claude session happens between cell `arm` and cell `disarm`, and a batch run would arm
and disarm within the same second. Run them by name, in order, and stop where the protocol says
to stop. (`--all` also executes `text` blocks, so the two prose cells below are excluded rather
than trusted to be inert.)

Nothing here depends on Runme. Every cell is a plain shell block and runs fine pasted into a
terminal; you lose the collected output, not the protocol.

**Read `docs/hostile-recording.md` first.** It is the original protocol and it still holds. This
file is the diff, not a replacement.

## What pass 2 left, and what changed since

| # | question | pass 2 |
| --- | --- | --- |
| 1 | Does a duration arrive? | **Answered.** It always did, as `duration_ms`; the adapter read `duration`. Fixed in `1b655ac`. |
| 2 | Shell failure vs tool failure | **Answered.** Both arrive on `PostToolUseFailure`; only `error` distinguishes them. |
| 3 | Does `PermissionDenied` fire for an interactive refusal? | **Missed.** The session ran in `auto` mode and the deletion was approved without a prompt. |
| 4 | What does interruption look like? | **Missed**, twice over. The harness refused `sleep 120` before dispatch, so there was nothing to interrupt — and the adapter was reading `interrupted` when the wire says `is_interrupt`, so the flag would have been dropped anyway. |
| 5 | Does `prompt_id` vary across turns? | **Answered, narrowly.** One value per turn, boundary where the human pressed enter. Necessary for a turn identifier, not sufficient. dragon:3 stays open. |
| 6 | *New.* Does `parent_agent_id` arrive? | Never observed — but the observation was taken from adapter output, which is exactly how question 1 went wrong. Never checked upstream. |

Question 6 is why this pass installs the probe on the subagent hooks as well. dragon:1 now
treats every "the integration never sent X" finding as unconfirmed until a raw payload says so,
and parentage is the biggest of them.

## Before you start

Three things must be true, and two of them were not true last time.

**The session must run in a permission mode that prompts.** `~/.claude/settings.json` currently
sets `defaultMode: "auto"`, which is what approved the deletion silently. The launch cell below
overrides it per-invocation rather than editing your settings.

**The interruptible command must be one the harness will dispatch.** A standalone `sleep` is
refused at the tool level and leaves no trace in any hook — verified in pass 2, in the recording
*and* in the independent probe. `python3 -c 'import time; time.sleep(120)'` dispatches normally.

**The outputs this collects are as sensitive as a recording.** `.witnessglass/pass-3/` will hold
raw hook key names, script output, and absolute paths. It is gitignored. It is not redacted, and
pass 2 established that neither the recording nor the probe capture is safe to attach anywhere —
the probe capture carries `cwd` and `transcript_path` on every line. See dragon:2.

## 1. Arm, install the probe, and write down the version

```sh {"name":"arm","excludeFromRunAll":"true"}
OUT=.witnessglass/pass-3
rm -rf "$OUT" && mkdir -p "$OUT"

{
  echo "### arm — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  scripts/arm.sh
  PROBE_HOOKS="PostToolUse PostToolUseFailure PermissionDenied SubagentStart SubagentStop" \
    scripts/probe.sh install
  echo
  echo "claude version: $(claude --version)"
  echo "witnessglass:   $(./target/debug/witnessglass --version 2>/dev/null || echo unknown)"
  echo "git commit:     $(git rev-parse --short HEAD)"
} 2>&1 | tee "$OUT/01-arm.txt"
```

The version line is not ceremony. dragon:1's first finding had to have its version reconstructed
from install metadata afterwards, and that only worked because nothing had overwritten it yet.

`PROBE_HOOKS` extends the probe past its usual three hooks so question 6 can be answered.
`scripts/probe.sh remove` takes out every probe block it finds regardless, so this does not need
undoing specially.

## 2. Run the session

This cannot be automated and should not be. Launch it yourself:

```sh {"name":"launch","interactive":"true","excludeFromRunAll":"true"}
claude --permission-mode default
```

Then, in that session, two submissions:

```text {"name":"submissions","excludeFromRunAll":"true"}
/hostile-1        … let it finish completely …
/hostile-3
```

`/hostile-3` is pass 2's turn-2 skill with the two broken steps replaced. If it does not exist
yet, create it from `/hostile-2` with these changes:

- **Step 3, the denial.** Keep `rm -rf /tmp/wg-probe/scratch`. In `default` mode this should
  prompt. **Choose no.** If no prompt appears, stop and record that — it means the mode override
  did not take, and the rest of the turn measures nothing about denial.
- **Step 4, the interruption.** Replace `sleep 120` with
  `python3 -c 'import time; time.sleep(120)'`. Wait about five seconds, then interrupt with Esc.

Both jobs are yours and neither can be scripted. Interrupting ends the turn; everything before it
is already recorded.

## 3. Disarm and read the raw payloads

```sh {"name":"disarm","excludeFromRunAll":"true"}
OUT=.witnessglass/pass-3
mkdir -p "$OUT"
{
  echo "### disarm — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  scripts/disarm.sh
  scripts/probe.sh remove
} 2>&1 | tee "$OUT/02-disarm.txt"
```

`disarm.sh` reporting that `settings.local.json` changed since arming and moving it aside is
correct, not an error — the probe edited it.

```sh {"name":"probe-show","excludeFromRunAll":"true"}
OUT=.witnessglass/pass-3
mkdir -p "$OUT"
scripts/probe.sh show 2>&1 | tee "$OUT/03-probe-show.txt"
```

Payload-quiet: key names and counts, never values. What to look for, in order of how much it
would change:

- **A `PermissionDenied` block at all.** If it is absent, the denial did not fire and question 3
  is still open regardless of what you clicked.
- **`parent_agent_id` in the `SubagentStart` key list.** Present means dragon:1's parentage
  finding was another adapter artifact. Absent — on a probe that shares no code with the adapter —
  is the first real evidence that the integration genuinely does not send it.
- **A duration-shaped key on `PermissionDenied`**, which no session has ever produced.
- **`is_interrupt` with a value on the interrupted call**, rather than merely present.

## 4. Read the recording

```sh {"name":"check","excludeFromRunAll":"true"}
OUT=.witnessglass/pass-3
mkdir -p "$OUT"
RECORDING="${RECORDING:-$(ls -t .witnessglass/recordings/*.ndjson 2>/dev/null | head -1)}"
{
  echo "### recording: $RECORDING"
  ./scripts/check-recording.sh "$RECORDING"
} 2>&1 | tee "$OUT/04-check.txt"
```

Newest recording by default. Set `RECORDING=` to name one explicitly. This is payload-quiet apart
from one documented exception: a corrupt record's parser diagnostic quotes the bytes it rejected.

```sh {"name":"summary","excludeFromRunAll":"true"}
OUT=.witnessglass/pass-3
mkdir -p "$OUT"
RECORDING="${RECORDING:-$(ls -t .witnessglass/recordings/*.ndjson 2>/dev/null | head -1)}"
{
  echo "### shape of $RECORDING"
  echo
  { printf 'seq\ttime\tprompt\tchannel\thook\tkind\ttool\tms\n'
    jq -r '[(.sequence|tostring),
          (.recorded_at|.[11:23]),
          ((.context.prompt_id // "-")|.[0:8]),
          .provenance.channel,
          (.provenance.mechanism|sub("command-hook:";"")),
          .event.kind,
          (.event.tool_name // "-"),
          ((.event.duration_ms // "-")|tostring)] | @tsv' "$RECORDING"
  } | column -t -s "$(printf '\t')"
  echo
  echo "distinct prompt_id values:"
  jq -r '.context.prompt_id // "(none)"' "$RECORDING" | sort | uniq -c
  echo
  echo "records by kind:"
  jq -r '.event.kind' "$RECORDING" | sort | uniq -c
} 2>&1 | tee "$OUT/05-summary.txt"
```

This prints tool names, timings, and mechanisms — not payloads. It is the one cell that goes
beyond what `check-recording.sh` will show you, and it is deliberately narrow about it.

The four questions, read off that table:

- **Denial** — a `tool_denied` record, on `command-hook:PermissionDenied`.
- **Interruption** — a `tool_failed` whose `interrupted` is `true`. Absent means not stated, which
  is a different claim from `false`.
- **Timing** — a `duration_ms` column that is no longer all dashes. This is the first recording
  written by an adapter that reads the right key.
- **`prompt_id`** — how many distinct values, against how many times you pressed enter. Two turns
  producing three values would be as informative as two producing two, and more interesting.

## 5. Look at it

```sh {"name":"view","interactive":"true","excludeFromRunAll":"true"}
RECORDING="${RECORDING:-$(ls -t .witnessglass/recordings/*.ndjson 2>/dev/null | head -1)}"
./target/debug/witnessglass view --recording "$RECORDING"
```

Serves on loopback until you stop it, so it is excluded from `--all`. The viewer's failure,
denial, and interruption rendering has still never met a real denial or a real interruption.
Anything it gets wrong is a defect worth fixing before the next recording, and the epistemic ones
outrank the cosmetic ones.

## 6. Collect

```sh {"name":"transcript","excludeFromRunAll":"true"}
OUT=.witnessglass/pass-3
mkdir -p "$OUT"
{
  echo "# Hostile recording, pass 3 — collected output"
  echo
  echo "Generated $(date -u +%Y-%m-%dT%H:%M:%SZ) from $OUT."
  echo "NOT redacted. NOT safe to share. See dragon:2."
  for f in "$OUT"/[0-9]*.txt; do
    echo
    echo "## $(basename "$f")"
    echo
    echo '```'
    cat "$f"
    echo '```'
  done
} > "$OUT/transcript.md"
echo "wrote $OUT/transcript.md"
```

One file, in order, with the commands and their real output. It replaces the paste-by-hand step
that produced pass 2's notes — which worked, and cost a round of "which of these outputs was
from which command".

## Recording the outcome

Findings go in the dragon they answer, never in this file:

- denial, interruption, timing on `PermissionDenied`, parentage → **dragon:1**
- `prompt_id` across turns → **dragon:3**
- anything about what the recording or the probe capture exposes → **dragon:2**

State the Claude Code version beside every finding. Cell 1 wrote it to `01-arm.txt` so this time
it does not have to be reconstructed.

**If a question misses again, the miss is the finding.** Pass 2's most useful result was not one
of its five questions — it was discovering that a harness-refused call leaves no trace in any
hook, which nobody thought to ask. Write down what did not happen.
