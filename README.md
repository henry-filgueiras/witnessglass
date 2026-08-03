# WitnessGlass

> WitnessGlass is a flight recorder for coding agents: declared intent, observed activity, and temporal replay.

## Status: experimental kernel plus a Claude adapter measured against one session

There is a working recording kernel, and a passive Claude Code command-hook adapter that
**has now recorded one real session** — one session, one macOS host, one Claude Code
version, 17 minutes, 234 records, structurally complete. What that session measured is
[section 3 of the adapter document](docs/claude-adapter.md); what it did not reach is
section 4, and section 4 is still the longer of the two. A surface the session did not
exercise — a tool failure, a permission denial, an interruption, a resume — is not a working
surface, and is not described as one.

What works today:

- an append-only UTF-8 NDJSON recording, one complete record per line, one file per session
- nine event kinds: session start and end, reported intent, tool requested, tool succeeded,
  tool failed, tool denied, subagent started, subagent stopped
- a passive `witnessglass claude-hook` adapter over eight Claude Code hook surfaces, which
  records and cannot influence the session it records
- tool lifecycle correlation through a stable `tool_use_id`
- deterministic replay in canonical append order, for both schema v1 and v2 recordings
- concurrent appends from independent short-lived processes, without a daemon or database
- explicit, tested behavior for unsupported and mixed schema versions, corrupt records,
  truncated tails, and ambiguous sequences
- a derived, receipt-bearing inspection projection over a replayed recording, in which every
  derived claim carries the raw sequence numbers supporting it and every count of zero carries
  the scope it was counted in
- `witnessglass view --recording <PATH>`: a foreground process that validates and projects one
  recording, holds it as one immutable in-memory snapshot, and serves it read-only to a browser
  on a loopback port behind an unguessable per-launch capability
- a local evidence workbench in that browser — three perspectives over an event map, a canonical
  event ledger, and an evidence inspector — in which every derived claim links back to the raw
  records supporting it

What does not exist: redaction, spans, summaries, or any second adapter.

**First contact is complete.** The first sprint closed with all seven of its success criteria
met against evidence, and its outcome — including where the recording and the recorded
session's own account of itself disagree — is in
[the sprint's archaeology](archaeology/sprints/0001-first-contact/sprint.md).

**The viewer has been run privately against the first-contact recording**, and reproduced every
finding task:4 had extracted by hand with `jq` — the 82 correlated request/completion pairs, the
`subagent_stopped` with no matching start, `duration_ms` absent on all 82 completions, parentage
absent on all three subagent records — without displaying the recording anywhere outside that
machine. What the viewer shows, derives, and cannot know is
[docs/viewer.md](docs/viewer.md); the sprint is
[sprint 2](archaeology/sprints/0002-first-light/sprint.md).

The raw-stream contract is
[decision 3](archaeology/decisions/0003-define-raw-stream-v1-and-canonical-replay-order.md),
refined by
[decision 4](archaeology/decisions/0004-represent-requested-and-effective-claude-tool-evidence-separately.md).

## Recording a Claude session

**Recording is opt-in and a clone records nothing.** Claude reads `.claude/settings.json`
and `.claude/settings.local.json`; both are gitignored. Only the inert example
`.claude/settings.witnessglass.example.json` is committed.

```sh
./scripts/arm.sh        # build, self-test the adapter, install the hooks
./scripts/disarm.sh     # remove them again
```

`arm.sh` rebuilds first, because the hooks invoke the built binary directly and a stale one
would quietly record a real session using old code. It then runs the adapter against a
synthetic payload and refuses to arm if that fails or if the adapter writes anything to
stdout. Re-running it while already armed re-arms from scratch. If you already have your own
`.claude/settings.local.json`, it is moved aside and `disarm.sh` puts it back; disarm never
deletes a file it did not write byte-for-byte, and never touches recordings.

Then start a **fresh** Claude session — arming mid-session produces a partial recording with
no session start. Recordings land in `.witnessglass/recordings/<session-id>.ndjson`, which is
gitignored and **is not safe to share**.

Afterwards, confirm the recording survived the session without putting it on screen:

```sh
./scripts/check-recording.sh .witnessglass/recordings/<session-id>.ndjson
```

That runs the recording through the same `replay`, discards every record, and keeps only the
verdict — exit 0 complete, 2 truncated tail, 1 corrupt or unreadable — with replay's one-line
summary left on stderr. Use it whenever the question is whether the flight recorder survived
the flight, since `replay` answers that by printing the whole flight. Payload silence has one
documented limit, described in the adapter document: a *corrupt* record's parser diagnostic
can quote the bytes it rejected, so a recording that checks as corrupt is the one not to
investigate on a shared terminal. Checking does not make a recording safe to share.

Read [docs/claude-adapter.md](docs/claude-adapter.md) before drawing any conclusion from a
recording. It states separately what Claude's documentation promises, what this adapter
maps, what one real session measured, and — at length — what is still unmeasured.

Measured against that one session:

- both session boundaries were captured, including the exit and its reason;
- 82 tool requests paired with 82 completions, with no unmatched record either way;
- a subagent's **own** tool calls were recorded, and are attributable to it by `agent_id`;
- `parent_agent_id` never arrived, so nothing links a subagent to what spawned it;
- one `subagent_stopped` arrived with no matching `subagent_started`;
- `prompt_id` arrived populated, but nothing in a recording says what it delimits;
- `duration_ms` never arrived, in any of 82 completions;
- a file written by a shell command left **no mutation event** — demonstrated, not asserted.

Still unmeasured, and not to be read as working:

- failure, denial, and interruption capture — none of the three was exercised;
- what a pre-tool record with no completion looks like in practice; none occurred;
- whether parallel dispatch is distinguishable from serial dispatch — it was not, here;
- whether a resumed session appends to the same recording;
- validation failures escaping the hooks entirely, and `@` references bypassing `Read`;
- under parallel hooks, `sequence` is recorder order, not causal order;
- total hook latency; only the append transaction itself has a number.

Scoped to macOS and Linux. Windows is untested and is not claimed to work.

## The name

In optical manufacturing, a *witness glass* is a sample coupon placed alongside a real
part and exposed to the same process — the same coating run, the same furnace, the same
bath. The part ships. The witness is kept, so that the process can be measured afterward
and the result traced back to the conditions that produced it.

That is the intent here. The agent does the work. WitnessGlass is the retained sample of
what the run actually consisted of.

## The problem

When an AI coding agent works on a repository, the durable output is a diff. The diff is
the residue of the process, not the process. It does not tell you what the agent believed
it was doing, which hypotheses it abandoned, what it ran, what failed, what it retried, or
in what order any of it happened.

Transcripts are not much better. They are long, unstructured, and mix the agent's own
narration with the effects of its actions, so the reader has to reconstruct causality by
reading prose. When something goes wrong — a bad edit, a test that was never actually run,
a claim of success that does not hold — the evidence needed to understand it is scattered
across chat text, shell history, and file mtimes, and is usually gone by the next session.

WitnessGlass is aimed at that gap: keep a session's evidence as it happens, in a form that
can be replayed in order and reasoned about later.

## Reported, observed, and derived

The central commitment is that these three are not the same kind of information and must
not be quietly merged.

**Reported** information is what the agent says. Intent, hypotheses, decisions, plans,
friendly descriptions of what it is about to do or believes it just did. This is
cooperative and semantically rich, and it is exactly the layer that a diff destroys. It is
also, by construction, a claim. An agent can report an intent it does not act on, describe
a test it never ran, or narrate a success that did not occur.

**Observed** information is what the surrounding machinery can see. Tool invocation and
completion, commands, exit status, file mutation, test execution. This is operationally
solid within its coverage and epistemically poor outside it: a process exit code is a fact
about a process, and says nothing about why anyone wanted it run.

**Derived** information is everything computed from the first two. Spans, timelines,
landmarks, findings, summaries, dashboards. These are projections. They are useful, they
are meant to be rebuilt freely, and they are never the record itself.

The invariants that follow:

- Raw session events are immutable and append-oriented: UTF-8 NDJSON, one complete record
  per newline-terminated line, one file per session.
- Reported intent is not promoted to ground truth because it sounds confident.
- Observed process facts are not promoted to intent because they are adjacent in time.
- Every event retains its source and fidelity, so a later reader can tell which channel a
  claim came from.
- Correlation may produce a view. It may not rewrite the evidence it correlated.

A recording that says "the agent claimed X and the process did Y" is more valuable than one
that has silently decided which of them was right.

## Cooperative hooks first, and the blind spots that implies

The primary sensor is intended to be cooperative: agent hooks, or an explicit semantic
emission API the agent calls. That is the only channel that can carry reported intent at
all, and it is the one that yields structured tool lifecycle instead of scraped text.

The cost is honest and worth stating up front. A cooperative sensor sees what the agent
chooses to tell it and what the integration surface happens to expose. It does not see
work done outside the instrumented path. OS- and process-level observation may later be
added to corroborate a recording, but v0 makes no claim that it can attach to an arbitrary
agent process and observe every descendant process — that promise is not portable, and
pretending otherwise would poison exactly the evidentiary value the project exists for.

Each adapter is therefore expected to document its fidelity and its blind spots
explicitly. "We did not see this" is a supported result. The Claude adapter's are in
[docs/claude-adapter.md](docs/claude-adapter.md), split between what one real session
measured and what is still provisional. The first recording produced a worked example of why
the split matters: the session's own account of what it did and the recording of what it did
disagree in two places, and both accounts are kept.

## Using the kernel directly

Every event below is synthetic, and here you are the emitter rather than the Claude adapter.

```sh
REC=/tmp/synthetic-session.ndjson

# The recorder's own boundary.
echo '{"session_id":"sess-synthetic-demo",
       "provenance":{"channel":"recorder","adapter":"manual","mechanism":"cli-stdin"},
       "event":{"kind":"session_started"}}' | witnessglass append --recording "$REC"

# What the agent says. A claim, recorded as a claim.
echo '{"session_id":"sess-synthetic-demo",
       "provenance":{"channel":"reported","adapter":"manual","mechanism":"cli-stdin"},
       "event":{"kind":"reported_intent","text":"Run the synthetic check.",
                "tool_use_id":"toolu_synthetic_demo"}}' | witnessglass append --recording "$REC"

# A request the machinery saw constructed. Same correlation id, different kind of
# claim — and note that this says nothing about whether the call ever ran.
echo '{"session_id":"sess-synthetic-demo",
       "provenance":{"channel":"observed","adapter":"manual","mechanism":"cli-stdin"},
       "event":{"kind":"tool_requested","tool_use_id":"toolu_synthetic_demo",
                "tool_name":"SyntheticTool",
                "requested_input":{"target":"/synthetic/example"}}}' | witnessglass append --recording "$REC"

# How it actually ended. The claim above said nothing about this, and vice versa.
echo '{"session_id":"sess-synthetic-demo",
       "provenance":{"channel":"observed","adapter":"manual","mechanism":"cli-stdin"},
       "event":{"kind":"tool_failed","tool_use_id":"toolu_synthetic_demo",
                "tool_name":"SyntheticTool",
                "effective_input":{"target":"/synthetic/example"},
                "error":"exit status 1"}}' | witnessglass append --recording "$REC"

witnessglass replay --recording "$REC"
```

That recording now holds a claim of intent next to an observed failure, correlated by
`toolu_synthetic_demo` and *not* reconciled into a single verdict. Preserving that
disagreement is the point.

That is four records. The third of them — the observed request — looks like this:

```json
{"schema_version":2,"session_id":"sess-synthetic-demo","sequence":3,
 "recorded_at":"2026-08-02T18:23:26.051104Z",
 "provenance":{"channel":"observed","adapter":"manual","mechanism":"cli-stdin"},
 "event":{"kind":"tool_requested","tool_use_id":"toolu_synthetic_demo",
          "tool_name":"SyntheticTool","requested_input":{"target":"/synthetic/example"}}}
```

Replay order is physical append order, carried by `sequence`. Timestamps are descriptive
metadata and are never sorted on, so a clock that jumps backwards mid-session cannot
reorder a recording. `replay` exits 0 when the recording is complete, 2 when it ends in a
truncated tail — the valid prefix is still printed, and the fragment is never presented as
an event — and 1 on corruption, an unsupported or mixed schema version, or an ambiguous
sequence.

A recording uses one schema version throughout. v1 recordings, written before the Claude
adapter existed, still replay; only v2 is written; appending across versions is refused at
both ends.

## Viewing a recording in a browser

```sh
witnessglass view --recording "$REC"            # opens a browser
witnessglass view --recording "$REC" --no-open  # prints the URL and waits
```

`view` replays and projects the recording *first*, so a corrupt one fails at the terminal
rather than in a tab, then binds a listener on an operating-system-selected port on
`127.0.0.1` and prints a URL carrying an unguessable per-launch capability. A truncated
recording is served: its valid prefix is evidence, and every absence in the projection is
scoped to that prefix rather than to a complete recording.

It reads the file once. The snapshot is held in memory and the file is never consulted again,
so changing or deleting the recording underneath a running viewer changes nothing it shows.
**It is not a daemon**: it runs in the foreground, watches nothing, and dies with the command.
Ctrl-C ends it and leaves no listener and no state behind.

Every response that could carry recording data requires the capability. A request without it
gets the same 404 as a request for a path that does not exist, carrying no session id, record
count, or schema version. Loopback binding is treated as one layer, not as the answer, and
there is no flag, environment variable, or configuration anywhere in this build that binds
anywhere else. Nothing about the request stream is logged, because the URL carries a secret
and the responses carry evidence.

The browser gets three perspectives — Events, Coverage, Provenance — with the investigative loop
in the first: an event map of point events, a canonical event ledger with search and filters, and
an evidence inspector beside it. Every derived claim
carries clickable receipts back to the raw records supporting it, and every count of zero carries
the scope it was counted in. Reported, observed, and derived are distinguished by a glyph and a
word, never by colour alone. **What it refuses to claim** — a turn count, a causal hierarchy, an
execution duration, a complete account of what a session changed, or the absence of failures — is
[docs/viewer.md](docs/viewer.md) §3.

Two synthetic fixtures are committed for trying it without a real recording:

```sh
witnessglass view --recording fixtures/synthetic-first-light.ndjson
witnessglass view --recording fixtures/synthetic-truncated.ndjson
```

**A rendered recording is exactly as sensitive as the recording.** Rendering is not redacting,
there is no export or share affordance, and there will not be one until a capture and redaction
contract exists.

## Privacy

Session recordings can contain source code, prompts, commands, absolute paths, command
output, and credentials. **The kernel redacts, filters, and omits nothing.** Everything an
emitter hands it goes into the recording.

Precisely: JSON values survive semantically — a string keeps its characters, a number its
value, an object its keys — but the stored record is not byte-identical to the emitter's
input. Whitespace, string escaping, numeric rendering, and object-key order may all be
normalized on the way in, before the bytes become recorded evidence. Nothing is dropped;
nothing is scrubbed. A credential handed to the recorder is a credential in the recording.

Recordings will not be described as safe to share until a concrete capture and redaction
contract exists, is implemented, and is tested. Real recordings are not committed here, and
every example and test fixture in this repository is synthetic. `.witnessglass/` is
gitignored so that a recording cannot be committed by accident, but that is a guard against
mistakes and not a safety property of the recording itself.

## Relationship to SignalScope

WitnessGlass descends from the thesis behind
[SignalScope](https://github.com/henry-filgueiras/SignalScope): that behavior is a story
over time rather than a snapshot, and that a system's history is the thing worth capturing.
SignalScope applied that to signals; WitnessGlass applies it to agent sessions. The lineage
is conceptual — this is a new codebase, not a fork.

The project used the working name *AgentScope* before this bootstrap. That name collides
with the established [AgentScope](https://github.com/agentscope-ai/agentscope) framework
and has been dropped; it survives only as historical provenance in the archaeology and is
not an alias for this project.

## Relationship to Scarp

Project archaeology — decisions, dragons, sprints, tasks — lives under `archaeology/` and
is managed with [Scarp](https://crates.io/crates/scarp). This is deliberate on two counts.

First, it is where the reasoning lives: the constraints above are recorded as decisions and
open uncertainties rather than as folklore, so a later reader (human or agent) can see what
was settled and what is still unresolved.

Second, WitnessGlass is a real external dogfooding case study for Scarp. Friction
encountered while doing genuine WitnessGlass work gets recorded as it is encountered, in
the task result where it happened. That is a byproduct of the work, not a reason to stop
the work and go modify Scarp.

Note the separation of concerns: product recordings are runtime data and belong nowhere
near `archaeology/`, which holds durable project knowledge.

## Non-goals for now

Not in scope at this stage, and not to be inferred from the framing above:

- a daemon, background collector, or distributed collection tier
- an MCP server
- a generalized plugin or adapter framework built before there are two real adapters
- AI-generated summarization of recordings
- attaching to arbitrary PIDs or OS-wide tracing
- multi-agent coordination
- publishing a crate
- redaction, export, or any notion of a shareable recording
- rotation, compaction, indexing, or streaming reads of large recordings

**Recently moved into scope:** derived projections — spans, timelines, landmarks, correlated
views — and a local presentation layer over them. Lifted by
[decision 5](archaeology/decisions/0005-lift-the-user-interface-non-goal-and-constrain-derived-projections.md)
once a working kernel and one real recording existed to project from. The projection and its
loopback server exist; the browser surfaces over them do not yet. Where the meaning of a
recording lives, and what a browser may therefore do with one, is
[decision 6](archaeology/decisions/0006-keep-recording-semantics-in-a-receipt-bearing-rust-projection.md).

That lift carries conditions, because a view is the surface where partial coverage is most
easily mistaken for complete observation. A projection must be rebuildable from the raw stream
and safe to delete; must keep reported and observed visibly distinct in the *rendering*, not
just in the data; must render absences as absences rather than filling them with plausible
values; must not group work by `prompt_id` while dragon:3 is open; and must stay local, with
no export and no implication that a rendered view is safer to share than the recording behind
it. Rendering is not redacting.

The lift is being spent on exactly one form: a foreground, loopback-only, read-only local
viewer over one explicitly supplied recording. The standing non-goals above are unchanged and
apply to it too — in particular the daemon, the AI summarizer, and redaction or export in any
form. The viewer adds its own, and they are not negotiable inside it:

- hosted access, remote binding, collaboration, accounts, or uploads
- download or shareable HTML
- live capture, file watching, tailing, or automatic refresh
- editing, annotation, bookmarks, or any mutation of a recording
- cross-session comparison or indexing
- a TUI, or a generalized frontend framework
- prompt or turn grouping, per
  [dragon 3](archaeology/dragons/0003-recorded-prompt-id-may-not-delimit-any-unit-of-work-a-projection-can-rely-on.md)
- inferred root agents, parentage, causality, concurrency, execution duration, or filesystem
  effects
- a flame graph or span hierarchy whose structure the evidence does not establish
- performance work for arbitrarily large recordings

## Development

```sh
./scripts/check.sh
```

That script is the gate, and CI runs the same script so local and CI semantics do not
drift. It runs:

- `bash -n` over `scripts/*.sh`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `scarp doctor`

You will need a Rust toolchain with `rustfmt` and `clippy`, and `scarp` on `PATH`
(`cargo install scarp --locked --version 0.2.0`).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed
as above, without any additional terms or conditions.
