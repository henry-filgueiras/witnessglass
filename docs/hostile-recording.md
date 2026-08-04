# Recording a deliberately hostile session

> ## HISTORICAL — pass 2's protocol, superseded. Do not run this as written.
>
> This ran on 2026-08-03 against Claude Code 2.1.220, and pass 3 (2026-08-04, 2.1.221)
> superseded it. Running it again as written reproduces both of its known misses: it does not
> set a permission mode that prompts, so nothing is denied, and its step 4 uses a bare `sleep`,
> which the harness refuses before dispatch so there is nothing to interrupt. Its `/hostile-2`
> command is marked historical for the same reasons.
>
> It is kept unedited because the questions below are what pass 2 set out to ask, and the
> reasoning that produced the answers is not recoverable from the answers alone. Read it as a
> record, not as instructions. **What is still open, and what a next pass would have to stage,
> is dragon:1 and dragon:3 — not this file.**

> **Three of its five questions are settled.**
>
> - **Question 1 is answered, and its premise was wrong.** The duration was never absent. The
>   integration sends `duration_ms`; the adapter read `duration` and discarded it unread from the
>   day it was written. Fixed in `1b655ac`.
> - **Question 2 is answered.** A non-zero shell exit and a tool-level error both arrive on
>   `PostToolUseFailure`. Only the `error` string distinguishes them, and it carries terminal
>   colour escapes.
> - **Question 5 is answered narrowly.** One `prompt_id` per turn, boundary where the human
>   pressed enter — necessary for a turn identifier, not sufficient. dragon:3 stays open.
> - **Questions 3 and 4 were missed**, both because of the session's setup rather than the
>   integration: it ran in `auto` mode so nothing prompted, and the harness refuses a standalone
>   `sleep` before dispatch so there was nothing to interrupt.
>
> The findings are in dragon:1, dragon:2, and dragon:3. **The next session runs
> [`hostile-recording-pass-3.md`](hostile-recording-pass-3.md)**, which fixes both misses and adds
> a sixth question about parentage.

Every recording this project has is of a session where nothing went wrong. 82 tool calls, 82
successes, no failure, no denial, no interruption, no resume, and — as far as anything here can
tell — one turn. Everything WitnessGlass does with failure, denial, interruption, and multi-turn
evidence is therefore exercised by synthetic fixtures only. The code renders those paths; no real
data has ever tested that it renders them *correctly*.

This is the protocol for fixing that. It is one session, two turns, about five minutes.

## What it is trying to measure

Five open questions, none of which needs a schema change or a line of code:

1. **Does `duration` ever arrive?** Absent on all 82 completions of the existing recording, on
   Claude Code 2.1.220, which `claude update` reports as current. `PostToolUseFailure` and
   `PermissionDenied` each document their own optional `duration` and **neither hook has ever
   fired here**. So the question is not settled by version — it is settled by hook.
2. **Does a non-zero shell exit produce `PostToolUseFailure`, or a `PostToolUse` carrying the
   error?** Unknown, and it changes how every recording should be read. Turn 1 provokes both a
   shell-level failure and a tool-level one so the two can be compared.
3. **Does `PermissionDenied` fire for an interactive refusal**, or only for the documented
   auto-mode classifier path? decision:4 recorded this as unmeasured.
4. **What does `interrupted` look like?** Never observed.
5. **Does `prompt_id` vary across genuinely separate turns?** dragon:3's first-order question.
   Two distinct values appeared across a 17-minute single-turn session, which explains nothing.
   Two real turns would.

## Why the session is attachable

Both turns confine the agent to `/tmp/wg-probe` and forbid it from touching the repository,
`git`, `$HOME`, or the environment. The adapter does not capture `cwd`, prompts, or transcript
paths. So a session run to this protocol should produce a recording whose payloads are entirely
synthetic scratch data.

**"Should" is not "does", and this project does not redact anything.** Read the recording before
attaching it anywhere. `scripts/probe.sh show` and `scripts/check-recording.sh` are both
payload-quiet and will tell you a lot without putting the file on screen; deciding it is safe to
share is a judgement only you can make, and nothing in this repository makes it for you.

For a bug report about `duration` specifically, you probably do not want the recording at all —
you want the probe output, which is a handful of raw payloads rather than a few hundred KB.

## The protocol

### 1. Arm, and install the probe

```sh
scripts/arm.sh
scripts/probe.sh install
claude --version        # write this down; the last finding lacked it
```

The probe is an **independent** observer: it shares no code with the adapter, parses nothing, and
appends each raw hook payload verbatim. That independence is the point — it can tell "the
integration did not send this" apart from "our adapter dropped it", which a field-population count
cannot.

### 2. Start a fresh session and run two turns

Arming mid-session produces a recording with no session boundary, so this must be a **new**
session.

```
/hostile-1        … let it finish completely …
/hostile-2
```

They must be **two separate submissions in one session**. A single combined prompt cannot answer
question 5, and restarting between them loses the comparison.

You have two jobs, both during turn 2, and neither can be scripted:

- **Deny** the deletion the agent asks permission for. If no prompt appears and it simply runs,
  that is itself a result — write it down.
- **Interrupt** the `sleep 120` after a few seconds. Interrupting ends the turn; everything before
  it is already recorded.

### 3. Disarm and read the result

```sh
scripts/disarm.sh
scripts/probe.sh show
./scripts/check-recording.sh .witnessglass/recordings/<session-id>.ndjson
witnessglass view --recording .witnessglass/recordings/<session-id>.ndjson
```

`disarm.sh` will report that `settings.local.json` changed since arming and move it aside rather
than delete it. That is correct — the probe edited it — not an error.

`scripts/probe.sh remove` takes the probe hooks back out if you left it armed; `clear` deletes
the captured payloads.

### 4. What to look at

- **`probe.sh show`** answers question 1 outright: it prints, per hook, whether a top-level
  `duration` arrived, and the full set of key names that did. Key names only — no values.
- **Which hook fired for the deliberate failures** answers question 2. Compare the `mechanism` on
  the resulting records: `command-hook:PostToolUse` versus `command-hook:PostToolUseFailure`.
- **Whether a `tool_denied` record exists at all** answers question 3.
- **Whether `interrupted` is populated** answers question 4, and the viewer surfaces it on the
  record's inspector panel.
- **Whether the tool calls in turn 1 and turn 2 carry different `prompt_id` values** answers
  question 5. The viewer shows `prompt_id` on each record and deliberately does not group by it,
  so this is a read rather than a filter.

The viewer's own failure, denial, and interruption rendering gets its first real test at the same
time. Anything it gets wrong is a defect worth fixing before the next recording, and the epistemic
ones take precedence over the cosmetic ones.

## Recording the outcome

Findings belong in the dragon they answer, not in this file:

- `duration`, hook-firing behaviour, and interruption → **dragon:1**, which already carries the
  first-contact coverage findings.
- `prompt_id` behaviour across turns → **dragon:3**, whose resolution criteria ask for exactly
  this observation, scoped to a stated version and host.
- Anything about what the recording exposes → **dragon:2**.

Whatever is concluded, state the Claude Code version beside it. The previous round's version had
to be reconstructed from install metadata after the fact, and that only worked because nothing had
overwritten it yet.
