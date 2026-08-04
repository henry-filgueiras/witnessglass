---
id: tsk_01KZ72SHJT9THQGFPMJ0KGFC1Y
sequence: 13
kind: task
status: closed
sprint: spr_01KZ50ZMMD79D5FS65F4CTV9C2
created: 2026-08-04
closed: 2026-08-04
---

# Calibrate the raw-payload probe and reconcile the pass-3 adapter contract

## Objective

Bring this repository's current-facing claims into line with what pass 3 actually measured, and
calibrate the instrument that produced those measurements so its own limits are stated rather
than implied.

Two halves, and they are related. Pass 3 moved the evidence boundary — failure capture is
exercised, an interactive denial fired no `PermissionDenied` hook and left a request with no
terminal record, `duration_ms` landed on every completion, parentage was independently confirmed
absent — while the README, `docs/claude-adapter.md`, and the module-level fidelity comments in
`src/claude.rs` still describe a repository that has recorded one session in which nothing went
wrong. Meanwhile the probe that produced the strongest of those observations is described in a
way that cannot be true: "fails when: never; it has no model to be wrong". Independence of
failure modes is not absence of them, and a probe that appends concurrent hook payloads into one
file with `cat >>` has a failure mode with a name.

Historical archaeology is not rewritten to look prescient. Later measurements are added beside
first-contact ones, not folded into them.

## Acceptance criteria

- Current official hook documentation is re-read, with the read date recorded, and stated
  separately from observed wire evidence. Where documentation and observation now agree, the
  agreement is stated as two independent facts rather than merged into one.
- README status, the measured/unmeasured summary, `docs/claude-adapter.md`, and the stale
  module-level fidelity comments in `src/claude.rs` describe the repository as it exists after
  pass 3, with each measurement carrying the session and version it came from.
- task:12 carries a dated pass-3 addendum recording that its seven-field inventory was
  superseded — which documentation-only entries were removed, which observed entries were added,
  and why — with the original Result intact above it.
- dragon:3 records the refreshed official `prompt_id` claim as a new claim to test. The dragon
  is not closed and the documentation is not treated as retroactive proof about `session_ended`.
- `/hostile-2` and `/hostile-3` are unmistakably marked historical and not to be rerun as
  written. No pass 4 is created or run here.
- The probe's raw capture is hardened against concurrent hook processes: one atomically
  completed file per hook invocation, raw bytes unparsed and unmodified inside each, a partial
  write distinguishable from a completed payload, `show` payload-quiet, `dump` explicitly
  sensitive, `clear` removing only what it owns and saying so, install and remove
  non-destructive, and legacy `raw-hooks.ndjson` captures still readable rather than destroyed.
- A synthetic concurrency regression test uses multiple large payloads — large enough that a
  shared `cat >>` would need several writes — and asserts every completed capture is
  independently parseable and that `show` prints none of the synthetic payload values.
- The probe's documentation states the narrower truth: its failure modes are independent of the
  adapter's rather than absent, `probe.sh show` is a parser and a summary rather than raw
  evidence, and completeness and parseability of a capture still have to be checked.
- Strict mode's actual granularity is documented: it detects a top-level field name absent from
  both the model and the deliberately-unrecorded registry, and does not detect a known field
  moving hook, an optional field disappearing, or a known field changing shape or becoming
  populated. Strict mode is not expanded into a schema framework here.
- `docs/viewer.md`'s "no aggregate is computed in the browser" is narrowed to what the
  architecture actually holds: JavaScript chooses no recording-semantic membership, grouping, or
  rollup, but does take the cardinality of Rust-supplied receipt sets and compute transient UI
  counts. The architecture is unchanged; the wording is corrected.
- `scripts/check.sh` passes, and focused searches find no surviving "one live session",
  "unexercised failure", "denial unmeasured", `model`/`stop_reason`-as-dropped, "the probe cannot
  fail", or "browser surfaces do not yet exist" claims.
- Nothing in this task changes the viewer's behaviour or the record schema, both of which
  sprint:3 excludes.

## Result

Delivered. Documentation, comments, and one shell tool changed; **no behaviour of the recorder,
the schema, the projection, or the viewer changed**, and `git diff src/` contains no non-comment
line, which is how sprint:3's fourth success criterion was kept rather than asserted.

### The documentation was re-read, and it had moved

<https://code.claude.com/docs/en/hooks>, **read 2026-08-04**; the previous reading in this
repository was 2026-08-02. The page was fetched whole rather than summarised, because the first
attempt came back truncated and an absent string in a truncated page is not evidence of absence
— which is the same mistake in miniature that this whole sprint exists to stop.

Four differences matter, and `docs/claude-adapter.md` §1 now carries them with the read dates
attached:

- `duration_ms` and `is_interrupt` are documented under the names that are actually delivered.
  The reference now agrees with the probe. **The probe is still what established it**, and the
  document says so in that order.
- `PermissionDenied` documents what it does *not* fire for: "it doesn't run when you manually
  deny a permission dialog". Pass 3 measured that first, independently, a day earlier.
- `parent_agent_id` and `parent_agent_type` **no longer appear in the reference at all**. The
  earlier reading recorded them as documented-and-optional. Documented-then-undocumented is not
  never-documented, so the old reading is preserved beside the new one rather than erased.
- `prompt_id` gained a meaning: "UUID identifying the user prompt currently being processed",
  correlated with an OpenTelemetry attribute. Recorded in dragon:3 as a **new claim to test**.
  The dragon is not closed, and the note says plainly why documentation cannot retroactively
  explain first contact's `session_ended` carrying a value no other record carried.

Two further current statements corroborate blind spots this project found by walking into them:
a validation rejection fires neither `PreToolUse` nor `PostToolUseFailure`, and a permission
denial fires `PreToolUse` and nothing after it. Both are recorded as corroboration, not as the
source.

### Current-contract reconciliation

- **README** — status is now three sessions rather than one; the measured list carries the pass
  each result came from, including the denial that fires nothing and the request left with no
  terminal record; the unmeasured list leads with interruption. Also fixed: "the browser
  surfaces over them do not yet [exist]", which had been false since sprint:2 closed.
- **`docs/claude-adapter.md`** — section 3 is now §3.1 first contact, §3.2 pass 2, §3.3 pass 3,
  each with its own date, version, and record count. **First-contact measurements are left as
  first-contact history**: the 82 completions with no duration stay absent, annotated as a fact
  about a recording written by a defective adapter, and the parentage finding keeps its original
  wording with a note that §3.3 later confirmed it upstream. Section 4 lost everything three
  sessions have since measured and gained what they did not reach.
- **`src/claude.rs`** — the module comment no longer ends "Nothing in this module has been
  exercised against a live session yet; that is task:4." Each fidelity bullet is now marked
  documentation or observation, because this module has twice been wrong by believing the first
  kind.
- **`docs/viewer.md`** — the denial rendering is named as the standing example of an unexercised
  surface, with what a reader actually sees for a denied call: a `tool_requested` that stops.
- **task:12** — a dated pass-3 addendum, with the original Result intact above it, recording
  that its seven-field inventory was superseded: `model` and `stop_reason` removed as
  documentation-only, four observed `SubagentStop` fields added, and the narrower claim that a
  quiet canary is quiet only about the hooks it was attached to.
- **`/hostile-2`, `/hostile-3`, and both protocol documents** — marked HISTORICAL in the
  frontmatter description, in a banner, and in an instruction to the invoked agent to say the
  protocol is superseded before doing anything. `/hostile-3` carries the sharper warning: its
  step 4 is *unreachable*, because denying at step 3 ends the turn. No pass 4 was created.

### Probe hardening

The capture is now **one atomically completed file per hook invocation**. `probe-hook.sh`
writes under `payloads/incomplete/` via `mktemp` and renames into `payloads/` only after stdin
is consumed without error, so a file in the spool is a whole payload and a file left in
`incomplete/` is a partial one — distinguishable without parsing either.

**The old shared-file capture was measurably broken, not theoretically.** Eight concurrent
512 KiB payloads fed through the previous `cat >>` implementation, in a scratch directory,
produced **four lines in total — two that parsed and two that did not — leaving six of the
eight payloads unrecoverable**. Claude runs matching hooks in parallel and pass 3's largest payloads were far
past a pipe buffer, so this was live during three sessions of measurement; nothing observed in
those sessions is known to have been affected, and nothing can now confirm that either.

Properties preserved, each with a test:

- raw bytes are unparsed and unmodified inside each capture — the newline the old hook appended
  when a payload lacked one is gone, since that was a modification, however small, to evidence;
- a partial write is distinguishable from a completed payload, and `show` counts them separately;
- `show` is payload-quiet: hook names, top-level key names, counts, duration-shaped key names,
  parse failures by file name. **Tool names are no longer printed** — they are payload values,
  and a summary that is safe on a shared screen cannot make an exception for the ones that look
  harmless. A count of distinct values replaces them;
- `dump` still announces itself as sensitive before printing anything;
- `clear` removes only the spool it owns, says how many completed and incomplete captures it
  removed, and **names any pre-spool `raw-hooks.ndjson` it deliberately kept**, with why;
- `install` merges into an existing configuration and `remove` takes out only probe blocks —
  verified by hand against a settings file carrying an unrelated `PreToolUse` hook and an `env`
  block, both of which survived a full install/remove cycle;
- a hook still installed with the **old single-file path** spools beside that file rather than
  appending into it, so an unmigrated settings file captures correctly and the old evidence is
  untouched.

`tests/probe_capture.rs` is the regression: eight concurrent hooks, 512 KiB each, asserting one
whole capture per invocation, each independently parseable, each byte-for-byte the payload its
hook was handed, none lost or duplicated, and `show` printing none of the sentinel values, no
tool name, and no session id.

### Exactly what strict mode and the probe still cannot do

Both are now written down where the claim they qualify lives, because the previous wording
claimed the probe "fails when: never; it has no model to be wrong".

**Strict mode detects one thing**: a top-level field name in neither `HookPayload` nor
`DELIBERATELY_UNRECORDED`. It does not detect a known field moving to another hook, an optional
field disappearing, a known field changing shape or type, a known field becoming populated where
it was always empty — the `background_tasks` case dragon:2 is watching — or anything nested. It
was not expanded, because expanding it means hand-maintaining a schema model of someone else's
payloads, which is what the flattened capture exists to avoid.

**The probe's failure modes are independent of the adapter's, not absent.** It can be installed
on the wrong hooks and capture nothing, lose a payload when a hook process is killed, and until
this task could corrupt its own capture under concurrency. `probe.sh show` is a parser and a
summary — the one part with a model of the payload — so the raw evidence is the captured files,
and completeness and parseability must be checked before any negative finding is drawn from a
run. `show` now prints all three numbers needed for that: completed captures, incomplete ones,
and captures that failed to parse.

One more: the second registry is hand-written and was wrong within three commits of being
written. "A field is listed only after being observed" is a convention enforced by review, not
by the compiler. `model` and `reason` sit unaccounted-for on purpose as its live test.

### Browser wording, narrowed rather than weakened

"No aggregate is computed in the browser" was false as stated: `viewer.js` takes
`coverage.present.records.length` and prints "N of M shown" under a filter. The architecture is
correct and unchanged; the sentence was not. Both `docs/viewer.md` §3 and the file's own header
now say the true thing — JavaScript chooses no recording-semantic membership, grouping, or
rollup, and does take the cardinality of a Rust-supplied receipt set and compute transient
interface counts. Counting a set somebody else defined is not deciding what belongs in one.

### Verification

- `cargo test --test probe_capture` — 6 tests, including the concurrency regression.
- `scripts/check.sh` — shell syntax, `cargo fmt --check`, clippy with `-D warnings`, the full
  test suite (`tests/workbench.rs` guards included, so the viewer comment edit did not break the
  source-level assertions), and `scarp doctor`: 29 artifacts, no problems.
- The old implementation was run against the same concurrent shape to confirm the regression
  test has teeth, and it failed as described above.
- `probe.sh install`/`remove`/`show`/`dump`/`clear`/`--help` were each run by hand in a scratch
  tree, including the empty-spool, unparseable-capture, and incomplete-capture paths.
- Focused searches for "one live session", "unexercised failure", "denial unmeasured",
  `model`/`stop_reason` as dropped-on-purpose, "no model to be wrong", and "browser surfaces …
  do not yet" return nothing except the sentence that explicitly corrects the probe claim.

### What this task did not do

No pass 4, no session recorded, no dragon closed. Interruption is still unobserved for the third
time, `PermissionDenied` has still never fired, and `permission_mode` is still dropped — the
schema decision dragon:1 argues for is still owed and is still not a quiet field addition.

### Desire-path friction

**Amending a closed artifact has no support at all, and leaves no trace.** This task appended a
dated pass-3 addendum to task:12, which was closed on 2026-08-03. Scarp has no command for it,
so it was `cat >> file.md` with the heading level hand-matched against a template nothing
validates — the same workaround idea:1 already covers for `## Result`, but with an extra edge
idea:1 explicitly declines to have an opinion on. The artifact now reads `closed: 2026-08-03`
and contains material written on 2026-08-04, and nothing in its front matter, its status, or
`scarp show` says so. `scarp doctor` passes, because it is checking identity rather than body
structure.

That matters more than the keystrokes. A tool whose value is a reviewable record of what was
known and when is silent about the one operation that changes what an artifact says after it was
finished. The pressure runs the wrong way: the cheapest correct-*looking* option is to edit the
original conclusion in place, which is the thing this workflow exists to prevent.

Promoted to **idea:4**, since it has now recurred across four rounds — three dragons extended
during sprint:1, three again after pass 2, two after pass 3, and a closed task plus an open
dragon here. The smallest useful affordance is `scarp amend <ref> --body-file <path>`: append
only, never rewrite, never touch lifecycle state, and stamp the artifact so a reader sees the
amendment without reading the prose.

**Everything else was frictionless.** `scarp new task --sprint sprint:3 --body-file` did exactly
the right thing, and `scarp new idea` likewise. The one genuinely improved thing since task:12
is that a sprint was already active, so filing this task cost one command instead of
commissioning a sprint to hold it.
