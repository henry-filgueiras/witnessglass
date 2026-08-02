---
id: tsk_01KZ2CDMWHMAW5ZYPD5KPBTB2V
sequence: 8
kind: task
status: closed
sprint: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
created: 2026-08-02
closed: 2026-08-02
---

# Project raw recordings into a receipt-bearing inspection model

## Objective

Build the pure Rust inspection projection that sits between a replayed recording and everything
the sprint renders, and record the raw/projection/browser boundary as an accepted decision once
that boundary is concrete and tested.

**Prerequisites: none.** This is the sprint's first task, and the other three depend on it.

The projection is a derived, disposable view under decision:5 — rebuildable from the raw stream,
never rewriting it, safe to delete. Its distinguishing property is that every derived entity
carries the raw sequence numbers supporting it, so a reader can always get from a claim back to
the records that licensed it. **A derived claim that cannot produce its receipts is asserting,
not deriving.**

What it may claim is fixed by task:4's measurements rather than by taste, and the temptation
this task has to resist is the pleasant one: filling a gap with a plausible value because the
gap is inconvenient to render.

## Acceptance criteria

- The projection is computed from a replayed recording and holds no fact the raw stream cannot
  regenerate. It never rewrites, overwrites, or tidies raw evidence, and discarding it loses
  nothing (decision:5, condition 1).
- Both the raw records and their canonical append sequence survive into the projection.
  `sequence` remains the only total order; no timestamp is sorted on (decision:3).
- Correlation is limited to relationships the evidence licenses, principally `tool_use_id` — the
  one identifier whose semantics this project has actually tested. Nothing derives parentage,
  causality, concurrency, or execution duration from containment, adjacency, or timing.
- Every derived tool lifecycle, anomaly, aggregate, and grouping retains the raw sequence numbers
  that support it.
- Paired and unpaired tool lifecycle evidence is represented without rejecting survivable
  anomalies. A request with no observed outcome, an outcome with no observed request, conflicting
  outcomes for one `tool_use_id`, and unmatched subagent boundaries are all first-class states
  rather than errors. decision:3 deferred pairing to a projection precisely so that a missing
  half is recorded as a blind spot; decision:4 made the fateless request first-class; task:4
  measured a real subagent stop with no observed start.
- Supplied agent attribution is represented exactly as supplied. An absent `agent_id` is
  "identity not supplied", never "root agent", and no parent identity is invented.
- Missing boundaries, unmatched requests and outcomes, conflicting outcomes, and unmatched
  subagent boundaries are detected and exposed rather than smoothed over.
- Channel, event-kind, adapter, mechanism, and supplied-agent aggregates are computed with
  wording that describes **records** rather than unseen reality: what was recorded, not what
  happened. Zero `tool_failed` records means no failure record was observed (task:4, dragon:1).
- v1, v2, complete, empty, and truncated recordings all project. v1's `observed_tool_started` is
  not silently equated with v2's `tool_requested`; decision:4 froze v1 exactly because the two
  mean different things. Truncation is carried into the projection as a first-class state.
- Prompt and turn grouping, and any execution-duration claim, are refused by construction rather
  than by convention (dragon:3; task:4 measured zero `duration_ms` across 82 completions).
- An accepted decision records the boundary: what the raw stream owns, what the projection may
  derive, what a browser may render, and — explicitly, as decision:5 asked rather than deferred
  by convenience — whether and how a projection may be served over a local HTTP port. It is
  written after the boundary is concrete and tested, and it does not restate the sprint plan.
- Focused tests cover, with synthetic fixtures only: equal and backward recorder timestamps;
  absent `duration_ms`; absent parentage; the duplicated reported description decision:4
  documents and task:4 measured 65-for-65; orphaned lifecycle records in both directions;
  conflicting outcomes for one `tool_use_id`; an unmatched subagent stop; and v1/v2 schema
  differences.
- The projection may be serializable for the local API, but it is not advertised as a stable
  public interchange format yet.
- No recording in `.witnessglass/` is read, listed, or copied by this task or its tests.
- `scripts/check.sh` passes, the slice is committed, and dragons 1–3 stay open.
## Result

`src/inspection.rs` is the sprint's projection layer, exposed as `witnessglass::inspection` with
`inspect` and `Inspection` re-exported at the crate root. It is one pure function of an already
validated `Replay`, 35 focused tests hold its boundary, and decision:6 records what that boundary
is. Nothing else changed: `Cargo.toml`, `Cargo.lock`, the CLI, the Claude adapter, both raw
schemas, replay, append, and the hook configuration are byte-for-byte unchanged, `publish` stays
`false`, and no task 9–11 work was started.

### 1. Shape

```
pub fn inspect<'a>(replay: &'a Replay) -> Inspection<'a>
```

Pure and total. It reads no file, consults no clock, and takes its input by shared reference, so
it cannot mutate what it derives from. It cannot fail: corruption is a replay failure upstream
and never reaches it, and every survivable irregularity inside a valid replay becomes projected
evidence or an anomaly rather than a projection error.

`Inspection<'a>` **borrows** the replay rather than copying it. That was a deliberate choice over
an owning model: "never rewrites raw evidence" becomes a property of the types instead of a
discipline, because the projection holds no raw bytes it could diverge from. The cost is that a
projection cannot outlive its replay, which is an accurate description of a derived view rather
than an inconvenience.

It preserves, per the acceptance criteria: `schema_version` including `None` for a recording with
no complete records; tail state; every raw `AnyRecord`, borrowed, in exact canonical order; every
raw sequence number; and timestamps as descriptive metadata only.

The public surface:

- `Inspection` — `schema_version`, `scope`, `session_id`, `records`, `ledger`,
  `session_boundaries`, `tool_groups`, `subagents`, `current_agents`, `aggregates`, `coverage`,
  `timestamps`, `anomalies`; plus `record_count()` and `tail()`.
- `Receipts` — ascending raw sequences, built only by scanning in canonical order.
- `ExaminedScope` — `CompleteRecording { records }` or
  `ValidPrefix { records, fragment_byte_offset, fragment_bytes }`. Tail state lives *here* rather
  than beside it, because an absence and the tail state it was found under are one fact.
  `Inspection::tail()` recovers the replay's own `Tail` from it.
- `RecordCount { records, scope }` — the unit every count travels in.
- `LedgerEntry` — one per raw record: sequence, `recorded_at`, raw `channel`, adapter, mechanism,
  schema-tagged `kind`, `correlation`, delivered `tool_name`, `current_agent`, `subject_agent`,
  `prompt_id`, `facets`, and the borrowed `record`.
- `PayloadFacets` — Rust-extracted field facts so a rendering layer can filter without
  reinterpreting raw event JSON: requested/effective input and response presence, delivered
  error, `duration_ms`, `interrupted`, v1 outcome, session source and reason, reported text.
- `ToolGroup`, `ToolEvidence`, `GroupShape`, `SequenceInterval`, `DeliveredValue`.
- `SubagentLifecycle`, `AgentAttribution`, `SubjectAgent`, `SuppliedParent`,
  `CurrentAgentAggregate`.
- `Aggregates`, `Tally`, `FieldCoverage`, `CoveredField`, `TimestampExtrema`, `TimestampPoint`.
- `Anomaly`, `AnomalyKind` — fourteen variants.
- `EventKind`/`V1Kind`/`V2Kind` and `CorrelationId`.

All of it derives `Serialize` and none of it derives `Deserialize`, which is the shape of an
output rather than an interchange format. The module documentation says in as many words that the
representation is internal and unstable and that nothing outside this repository may depend on it.

Output ordering is deterministic throughout. Groups, subagents, and anomalies sort by their
earliest supporting record; string-keyed tallies keep first-appearance order via a small
`FirstAppearance` helper rather than a hash map; channels use a fixed vocabulary order. A test
serializes two projections of one replay and compares the strings.

### 2. Receipts and negative evidence

Every derived entity carries the raw sequences supporting it. There is no constructor that
produces a derived claim without them.

The harder half was negative claims, and the answer is that **an empty receipt list is never a
receipt on its own**. Counts travel as `RecordCount`, which is the matching sequences *plus* the
population searched. That makes three statements distinguishable and a fourth unstatable:

- "no matching record in this complete recording" — `CompleteRecording { records: n }`;
- "no matching record in the valid prefix of this truncated recording" — `ValidPrefix { .. }`;
- an anomaly that *is* an absence (`MissingSessionEnd`) carries empty receipts and a scope, which
  is the only thing that makes it readable;
- a claim about what happened outside recorded evidence — the model cannot express one.

`ExaminedScope` is deliberately the smallest thing that does this. There is no proof engine, no
predicate language, and no provenance graph: two enum variants and a record count.

Schema-aware zero counts fall out of the same idea. `aggregates.by_event_kind` enumerates the
recording's **whole** schema vocabulary — nine kinds for v2, five for v1 — so a kind the recording
contains none of appears as a zero tally with its scope attached. That is how task:4's finding
gets said: zero `tool_failed` records means no failure record was observed, and the scope says
where it was looked for. For a recording with no complete records the vocabulary list is empty,
because no schema was ever declared and enumerating one would be choosing a schema on the
recording's behalf.

Field coverage is stated the same way, as `population` / `present` / `absent` triples with
receipts, for `duration_ms` (over v2 completions and failures), `interrupted` (over v2 failures),
supplied parent identity (over v2 subagent boundaries), and `prompt_id` presence (over all v2
records). Every one means "records observed", never "events that occurred".

### 3. v1 and v2 correlation

`CorrelationId` is `V1ToolCallId(&str) | V2ToolUseId(&str)`. Because it is a tagged enum, two ids
spelled identically under different schemas are different map keys and compare unequal — accidental
cross-version equivalence is unrepresentable rather than merely discouraged. A test asserts it
directly.

`ToolEvidence` is likewise two variants that never meet:

- `V1 { started, finished_succeeded, finished_failed }` — `started` records v1's claim that a call
  was witnessed *beginning*, which v2 refuses to make, and the outcome vocabulary is v1's
  succeeded/failed with no denial.
- `V2 { requested, succeeded, failed, denied }` — a request is not an execution, and denial is not
  failure.

v1's lack of v2's causal context is preserved as a third state rather than collapsed into
"absent": a v1 record's attribution is `AgentAttribution::NotRepresentable`, because the v1
envelope has no context field at all, which is a different fact from asking and getting nothing.
v1 records carry no `prompt_id` and contribute to no v2 coverage population.

Reported intent is correlated and never fused. It stays a separate record on the `reported`
channel with its own receipts inside the group, counted separately from observed evidence. This
covers the duplication decision:4 documents and task:4 measured 65-for-65: the description
appears in the requested input *and* as its own reported record, and the projection files the
second as a claim beside the first rather than as a second observation. Nothing reconstructs
intent from a command, tool name, payload description, path, temporal proximity, or result.

### 4. Cardinality and anomalies

Nothing is greedily paired. Evidence is grouped by correlation id and classified:

| `GroupShape` | Meaning |
| --- | --- |
| `ReportedIntentOnly` | a claim citing an id no observed record carries |
| `OpeningWithoutOutcome` | request or claimed start, no outcome in scope — not "still running" |
| `OutcomeWithoutOpening` | outcome with no opening, the only evidence the call existed |
| `PairedLifecycle` | exactly one of each — the only shape describable as a paired lifecycle |
| `Ambiguous` | duplicates on either side, or outcomes that disagree |

`SequenceInterval { opening, outcome }` exists only for `PairedLifecycle`, and its documentation
states what it is not: not elapsed time, not execution duration, not nesting, not causal
containment, and records falling between the two positions are not thereby children of anything.

The fourteen `AnomalyKind`s: missing and duplicate session start and end, duplicate openings,
duplicate outcomes, conflicting outcomes, opening without outcome, outcome without opening,
reported intent without observed evidence, divergent tool names, subagent stop without start,
subagent start without stop, and divergent agent types. Each carries receipts and scope.

Two details worth recording. A missing half is reported on its own terms independently of the
shape classification, so two requests and no outcome produces *both* `DuplicateOpenings` and
`OpeningWithoutOutcome` rather than hiding the second behind the word "ambiguous". And duplication
is kept distinct from conflict: two successes for one id are duplicated and disagree about
nothing, so no `ConflictingOutcomes` is raised. Divergent delivered fields — two tool names for
one id — are kept as `DeliveredValue`s with receipts and no canonical pick, and do not change the
cardinality, which stays one-to-one.

### 5. Agent identities

Three identities, structurally separate, with no path between them:

- **Current agent** — `context.agent_id`/`agent_type`, the agent a record was *delivered from*.
  `AgentAttribution::Supplied | NotSupplied { agent_type } | NotRepresentable`.
- **Subject agent** — a `subagent_started`/`subagent_stopped` event's own `agent_id`, the child
  the event is *about*. Filed in `SubjectAgent`, never used as the emitter's identity.
- **Supplied parent** — present only when the event delivered `parent_agent_id` or
  `parent_agent_type`. `supplied_parent` is `None` when neither arrived, and no code path
  constructs one from anything else.

Absent identity is "not supplied" and never "root" or "main". `CurrentAgentAggregate` keeps
`supplied` (per delivered id), `not_supplied`, and `not_representable` apart, and subject ids are
deliberately *not* folded into the attribution tallies — a boundary record about a child is not
evidence that the record came from it. Where one id arrives with inconsistent types, every
delivered value is retained on its record and a `DivergentAgentTypes` anomaly carries the
receipts; the first is not selected.

The containment case task:4 measured is tested explicitly. A recording with subagent-attributed
records between an `Agent` call's request and outcome produces a `PairedLifecycle` for the `Agent`
call, a `PairedLifecycle` for the inner call, an empty `supplied_parents`, and no relationship
whatsoever between them. Containment in the append chain produces no parent, no child, and no
nested span.

`prompt_id` survives on every ledger entry and in one presence-count coverage summary. It defines
no group. A test builds a recording whose two records carry two different `prompt_id` values and
asserts the projection still contains exactly one tool group — the identifier did not segment
anything. Nothing in the codebase counts turns.

### 6. Tests and gate

`tests/inspection.rs`, 35 tests, synthetic fixtures only. `tests/common/mod.rs` gained bare-event
builders for both schemas, a `context` helper, and `v2_recording`/`v1_recording`, which assign
canonical sequences and render NDJSON — every fixture goes through real `replay_bytes` validation
rather than constructing a `Replay` by hand. No new dependency.

Coverage, against the task's list and the prompt's: v1 correlation through `tool_call_id` without
adopting v2 semantics; v2 correlation through `tool_use_id`; a v1 id and a v2 id spelled the same
not being one key; reported intent staying separate; request-only, outcome-only, and intent-only
groups; duplicate requests and duplicate outcomes not greedily paired; success, failure, and
denial distinct; conflicting outcomes in both v1 and v2 with all receipts; divergent tool names as
delivered evidence; current-agent attribution distinct from a subagent event's child id; absent
identity unattributed; supplied parent retained and absent parent never inferred; two types for
one agent id, both for a current agent and for a subagent subject; sequence containment inside an
`Agent` call producing no parentage; empty complete recordings; truncated recordings with no
complete record; truncated v1 and v2 recordings; absences carrying valid-prefix scope; zero counts
across the whole vocabulary with scope; absent `duration_ms` as coverage; `prompt_id` presence
without grouping; equal and backward timestamps leaving append order intact and extrema ties
citing the earliest record; missing and duplicate session boundaries; unmatched subagent starts
and stops; every receipt in a deliberately anomalous eight-record recording naming a real
sequence; deterministic ordering by serialized comparison; and projecting without mutating the
replay, including a re-render byte-comparison against the original recording.

No test invokes the CLI, a real Claude process, a browser, or a real recording. `.witnessglass/`
was not listed, opened, read, or referenced at any point in this task.

`./scripts/check.sh`, final run:

```
==> shell syntax
==> cargo fmt
==> cargo clippy
==> cargo test
    0, 0, 11, 18, 27, 2, 18, 35, 9, 8, 7 passed; 0 failed
==> scarp doctor
doctor: 25 artifact(s) checked, no problems found
==> all checks passed
```

135 tests, up from 100. The 35 new ones are the inspection suite; the other 100 are unchanged and
still pass, which is the check that mattered most — the projection was added beside the kernel and
did not touch it.

### 7. decision:6

decision:6, *Keep recording semantics in a receipt-bearing Rust projection*, accepted, written
after the model and its tests made the boundary concrete. It records raw replay's authority; the
projection's authority and its receipt obligation; schema-specific correlation rather than
flattened lifecycles; the browser as renderer rather than semantic interpreter; derived claims as
projection-level objects rather than a fourth raw `Channel`; no persistence requirement and no
stable public schema; survivable anomalies as evidence rather than parse failures; why a negative
claim needs an explicit examined scope; and why the three agent identities stay separate.

It answers decision:5's deferred question narrowly. Yes: task:9 may serve exactly one immutable
projected snapshot through its planned foreground, capability-protected loopback process, and the
browser receives the projection rather than raw NDJSON. No: no persistence, watching, remote
binding, or lifetime beyond the invocation. And explicitly — HTTP, capability handling, headers,
browser launching, and every security property of that transport are unimplemented and unverified
until task:9 does them. An accepted architectural constraint may precede its implementation; a
test result may not, and decision:6 says so.

### 8. Two archaeology precision corrections

Review found two phrases looser than task:4's evidence. Both are appended as follow-ups; neither
historical conclusion was rewritten or reopened.

**decision:5** described the subagent's 27 tool calls as "nested inside the parent's single
`Agent` call". The appended follow-up narrows that to what was measured — the subagent-attributed
records fell *between* the `Agent` call's request and outcome in append sequence — and states that
containment in the append chain establishes no causal parent, no parent agent, and no nested span.
decision:5's own third condition already forbids inferring parentage from containment; the
correction brings the evidence cited for the lift down to the strength of the constraint imposed
by it.

**sprint:1's outcome** said the recording "contains no interleaving whatsoever". The appended
follow-up narrows that to: no overlapping *sibling* tool request/outcome intervals; one containing
interval, the `Agent` call's, whose meaning the evidence does not settle; and parallel versus
serialized dispatch still unobservable. The finding it supports is unchanged.

Both point back at task:4. Neither is new empirical evidence, and neither touches a dragon.

### Scarp desire paths

**idea:1 recurred, for the eighth time.** This result was written to a temporary file and appended
with a shell redirect before `scarp close task:8`. Eight for eight. Nothing new to say about it.

**A near-miss on idea:1's shape, worth one line.** The two precision corrections above are the
same operation applied to *closed* artifacts — write prose to a temp file, append with a redirect
— and Scarp has no command for that either. It is deliberately not filed as a new idea: appending
to a closed artifact is rare, is correctly rare, and an affordance that made it convenient would
be an affordance for rewriting history. The friction here is doing its job.

**No new idea is filed.** `scarp new --body-file` with `## ` headings matching the decision
template worked first time and produced decision:6 with no fixups, which is the workflow idea:2
was originally about and which now has a closed artifact of the same kind to read from. `scarp
doctor` was run after each archaeology change and caught nothing, because there was nothing to
catch. The real friction in this task was Rust lifetime plumbing in a test helper, which is not a
Scarp problem.

### One concern for task:9

The projection borrows its `Replay`, so task:9 must keep the replay alive for as long as it holds
the projection, or serialize the projection to a `String` once at startup and serve that. The
second is likely what it wants anyway — sprint:2 promises exactly one immutable snapshot — and it
is the shape that makes "immutable snapshot" true rather than merely intended. Recomputation on
demand is not a requirement anywhere and would be a worse fit for a process that reads one
recording once.
