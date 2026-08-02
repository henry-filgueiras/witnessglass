# WitnessGlass

> WitnessGlass is a flight recorder for coding agents: declared intent, observed activity, and temporal replay.

## Status: experimental kernel; no agent adapter yet

There is now a working recording kernel. It can append events to a session recording and
replay them deterministically. It cannot yet be pointed at Claude, or at any other agent,
because **no adapter exists** — nothing hooks WitnessGlass up to a real session. Events
come from whoever runs the CLI or calls the library.

What works today:

- an append-only UTF-8 NDJSON recording, one complete record per line, one file per session
- five event kinds: session start and end, reported intent, observed tool start, observed
  tool finish
- tool lifecycle correlation through a stable tool-call ID
- deterministic replay in canonical append order
- concurrent appends from independent short-lived processes, without a daemon or database
- explicit, tested behavior for unsupported schema versions, corrupt records, truncated
  tails, and ambiguous sequences

What does not exist: any Claude integration, redaction, projections, spans, timelines,
summaries, or a UI. The full contract is written up in
[decision 3](archaeology/decisions/0003-define-raw-stream-v1-and-canonical-replay-order.md).

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
explicitly. "We did not see this" is a supported result.

## Using the kernel

Every event below is synthetic. There is no adapter, so nothing produces these
automatically yet — you are the emitter.

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
                "tool_call_id":"toolu_synthetic_demo"}}' | witnessglass append --recording "$REC"

# What the machinery saw. Same correlation id, different kind of claim.
echo '{"session_id":"sess-synthetic-demo",
       "provenance":{"channel":"observed","adapter":"manual","mechanism":"cli-stdin"},
       "event":{"kind":"observed_tool_started","tool_call_id":"toolu_synthetic_demo",
                "tool_name":"SyntheticTool",
                "arguments":{"target":"/synthetic/example"}}}' | witnessglass append --recording "$REC"

# How it ended. The claim above said nothing about this, and vice versa.
echo '{"session_id":"sess-synthetic-demo",
       "provenance":{"channel":"observed","adapter":"manual","mechanism":"cli-stdin"},
       "event":{"kind":"observed_tool_finished","tool_call_id":"toolu_synthetic_demo",
                "outcome":"failed","result":{"exit_status":1}}}' | witnessglass append --recording "$REC"

witnessglass replay --recording "$REC"
```

That recording now holds a claim of intent next to an observed failure, correlated by
`toolu_synthetic_demo` and *not* reconciled into a single verdict. Preserving that
disagreement is the point.

That is four records. The third of them — the observed start — looks like this:

```json
{"schema_version":1,"session_id":"sess-synthetic-demo","sequence":3,
 "recorded_at":"2026-08-02T18:23:26.051104Z",
 "provenance":{"channel":"observed","adapter":"manual","mechanism":"cli-stdin"},
 "event":{"kind":"observed_tool_started","tool_call_id":"toolu_synthetic_demo",
          "tool_name":"SyntheticTool","arguments":{"target":"/synthetic/example"}}}
```

Replay order is physical append order, carried by `sequence`. Timestamps are descriptive
metadata and are never sorted on, so a clock that jumps backwards mid-session cannot
reorder a recording. `replay` exits 0 when the recording is complete, 2 when it ends in a
truncated tail — the valid prefix is still printed, and the fragment is never presented as
an event — and 1 on corruption, an unsupported schema version, or an ambiguous sequence.

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
every example and test fixture in this repository is synthetic.

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
- an MCP server, TUI, web UI, or dashboard
- a generalized plugin or adapter framework built before there are two real adapters
- AI-generated summarization of recordings
- attaching to arbitrary PIDs or OS-wide tracing
- multi-agent coordination
- publishing a crate
- derived projections — spans, timelines, landmarks, findings
- redaction, export, or any notion of a shareable recording
- rotation, compaction, indexing, or streaming reads of large recordings

## Development

```sh
./scripts/check.sh
```

That script is the gate, and CI runs the same script so local and CI semantics do not
drift. It runs:

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
