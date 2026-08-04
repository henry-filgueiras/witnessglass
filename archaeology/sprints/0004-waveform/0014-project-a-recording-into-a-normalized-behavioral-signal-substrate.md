---
id: tsk_01KZ75SJCNCS42RMG2739CGSTC
sequence: 14
kind: task
status: closed
sprint: spr_01KZ75RKADD2ENBZR202PFZ1RF
created: 2026-08-04
closed: 2026-08-04
---

# Project a recording into a normalized behavioral signal substrate

## Objective

Build the normalized multivariate behavioral signal substrate sprint:4 exists to test, and validate
it against a deterministic synthetic recording whose hidden structure is known before the projection
runs.

The deliverable is a substrate, not an analysis. Nothing here looks for a motif, a regime change, or
a scale. It establishes whether the numbers a detector would be handed are honest, regular, and
recoverable from raw — and it establishes that with a fixture built so that the right answer is
known in advance and the projection can be caught being wrong.

Two things make this harder than the arithmetic suggests, and both are the reason it is worth doing
carefully once:

- **The time axis does not exist yet.** `sequence` is the canonical order and is not a clock;
  `recorded_at` is a clock and is explicitly not the order. A bucketed series needs a clock, so it
  has to use `recorded_at`, has to say so, and has to state what that costs rather than quietly
  spending it.
- **The obvious dimensions are not licensed.** "Read activity", "search activity", and "compiler or
  test activity" are semantic categories over delivered tool-name strings, and no decision in this
  repository establishes that mapping. "Unique files touched" requires reading meaning out of an
  input payload the schema stores uninterpreted, which is the filesystem-effect inference
  `CLAUDE.md` §6 names directly. Each of those has to be refused in the code, not just avoided.

## Acceptance criteria

- A `witnessglass::experiment` module, documented in its own header as a disposable sprint:4
  research experiment, consumes an already-validated `Inspection` and produces a contiguous,
  evenly spaced multivariate series. It reads no file, consults no clock, writes nothing, and
  borrows rather than owns, so it cannot rewrite what it derives from.
- The dimension set is derived from the recording rather than declared in advance, and every
  dimension is one of: a record count, a raw provenance channel count, a schema-tagged event-kind
  count, a count for one verbatim delivered tool-name string, a count of distinct delivered
  correlation ids, or a directly measurable magnitude of a recorded value. No dimension classifies
  a delivered string into a category, and none reads meaning out of an uninterpreted payload.
- The refusals are written down beside the definitions: which candidate dimensions were considered,
  which were rejected, and which specific piece of evidence each would have required and did not
  have.
- Each sample carries the raw sequence numbers of the records that produced it, so a bin's numbers
  can be traced back to evidence, and an empty bin carries an empty receipt list plus the examined
  scope, in the shape decision:6 already requires of every other derived claim.
- Defined and tested behaviour for: empty bins, the first and last bin, the partial final bin,
  sparse dimensions, zero-variance dimensions, a truncated recording's valid prefix, a recording
  with no records at all, a recording whose whole span is shorter than one bin, and timestamps that
  move backwards.
- Bucket width is a parameter with a documented default. No configuration file, no environment
  variable, no settings type.
- One normalization policy, chosen against the measured shape of this data and not by reflex,
  documented with the reason the alternative was rejected, deterministic for a zero-variance
  dimension, and derived rather than destructive: the unnormalized counts survive alongside it.
- A committed synthetic oracle fixture holds a known structure — baseline, a repeated motif,
  baseline, a regime change, an elevated regime, and a recurrence of the motif with deterministic
  noise — built only from the v2 vocabulary as it actually is, obviously synthetic in every record,
  and regenerable byte-for-byte from committed code so it cannot drift from the structure it claims.
- Tests assert the properties the fixture encodes: bin count and spacing, dimension identity and
  order, motif periodicity and where it lands, the regime boundary, receipt totals reconciling
  against the record count, and the normalization invariants — mean zero, unit variance, and the
  zero-variance rule. The tests check the substrate, not a detector.
- One invocation surface, and it is not the product CLI: an example binary that prints a summary
  and, on request, the samples. `witnessglass --help` gains nothing.
- No new dependency. `scripts/check.sh` passes unweakened, and no existing test is changed to
  accommodate this work.

## Result

Delivered, and the substrate holds. A validated recording projects into a regular, evenly spaced,
normalized multivariate series in which every deliberately injected structure survives and is
measurable. **No detector was implemented**, and the sprint's question — whether an algorithm then
*finds* that structure — is still open on purpose.

Everything below is one module, one example, one fixture, one test file, and two lines in
`lib.rs`. `git rm` on those removes the experiment and leaves the crate as sprint:3 left it.

### 1. What was implemented

`witnessglass::experiment::signal` turns an already-validated `Inspection` into a
`BehavioralSignal`: a dense `T × D` matrix of `f64`, `T` contiguous evenly spaced buckets by `D`
discovered dimensions, plus the axis it was bucketed on and the caveats attached to that axis.

It consumes the projection rather than raw NDJSON, which is decision:6 applied to a second
consumer: `inspection` is the one place recording semantics live, and a substrate that parsed the
stream itself would be the second opinion that decision exists to prevent. It reads no file,
consults no clock, borrows rather than owns, and is a pure function of `(Inspection, BucketWidth)`
— asserted, not asserted-about: two projections of one replay compare equal and the replay is
unchanged by either.

Each sample carries the ascending sequence numbers of the records that produced it. That is
decision:6's receipt discipline applied to a bucket: every number in a row can be traced to the
records behind it and recomputed from them, and the test that reconciles all receipts against
`1..=196` is what makes "each record placed exactly once, by its own timestamp" a checked property
rather than a hope.

`Inspection::Receipts` could not be reused for those, because it has no public constructor that
takes values and `push` is private to `inspection`. Rather than open the production type up for an
experiment, a sample's receipts are a plain `Vec<Sequence>` and the examined scope is carried once
on the signal instead of per row. Not free — a sample's empty receipt list has to be read against
`BehavioralSignal::scope` one level up rather than carrying its own — and the alternative was
editing production code to suit disposable code.

### 2. Dimensions, and what licenses each

Twenty for the real recording, nineteen for the oracle. Six families, every one a count or a
measurement of something literally in a record:

| Family | Licensed by |
|---|---|
| `records` | one ledger row is one raw record |
| `channel:{reported,observed,recorder}` | `provenance.channel`, a raw enum field |
| `kind:v{1,2}:*` | the record's own `kind` tag, schema-tagged by `inspection` so v1 and v2 never merge |
| `tool_name:<delivered>` | `tool_name` **verbatim**, byte for byte as delivered |
| `distinct_correlation_ids` | delivered `tool_use_id`/`tool_call_id`, the identifiers decision:6 licenses |
| `recorded_response_json_bytes` | the serialized size of the response value this recording holds |

The dimension *set* is discovered from the recording, not declared. The whole event-kind
vocabulary appears including kinds the recording contains none of, so `kind:v2:tool_denied` is a
stated zero rather than a missing column — `inspection`'s zero-tally convention carried into a
matrix. Two recordings generally have different shapes; comparing them numerically would need an
alignment step that does not exist, and that is stated rather than papered over.

**What was refused, and why.** These are in the module header beside the definitions, and one test
asserts them behaviourally rather than trusting the prose:

- **"shell / read / search / edit activity"** — needs a map from a delivered string to a semantic
  category. No decision here establishes one and the strings come from the integration, not from
  this project. A per-tool-name column is a partition by a delivered value, which is much weaker
  and is what got built. A reader who knows the integration may group columns afterwards.
- **"compiler or test activity"** — the same, but classifying the *contents* of a command string.
  Precisely the promotion `CLAUDE.md` §2 forbids.
- **"unique files touched"** — needs a path read out of an input payload every schema stores
  uninterpreted, and then presents a tool-derived list as an account of what changed on disk.
  decision:5's third condition, `CLAUDE.md` §6, and first contact's own counterexample — files
  changed by a shell command with no mutation event anywhere — all point the same way.
- **"output volume"** — not directly observed. What is observed is the size of the response value
  *the recording holds*, after JSON normalization, which is a fact about the recording rather than
  about what a tool emitted. The dimension is named for that narrower thing.
- **`duration_ms`** — supplied on some completions and absent on others, and absent is not zero. A
  matrix column would need a filler for the absences, which is the failure decision:5 calls the
  one most likely to be broken by accident. `inspection` can report its coverage honestly; a
  column cannot.
- **Anything segmented by `prompt_id`** — dragon:3 is open.

The test builds a recording delivering `Bash`, `Read`, and `Grep` with a `cargo test` command and a
`/x/y.rs` path in the payloads, and asserts the substrate produces exactly three verbatim columns
and no column whose own vocabulary contains *shell*, *search*, *compiler*, *test*, *file*, *path*,
*duration*, or *prompt*.

### 3. Bucket width and normalization

**Width: 500 ms by default, a parameter, and deliberately not a decision.** `BucketWidth` is a
newtype that refuses zero once at the boundary. 500 ms resolves the sub-second structure a hook
adapter produces around one call — the gap between a request record and its outcome is tens to
hundreds of milliseconds, and a wider bucket collapses them into one number.

It is a *bad* width for looking at a whole session, and that is measured. See §7.

**Normalization: per-dimension z-score against the population mean and standard deviation**, `N`
rather than `N-1`, because the signal is the whole finite series being described and not a sample
from a larger one. Derived and additive: `normalize(&self)` returns a new value, the unnormalized
counts are never touched, and a caller always holds both. Pure — normalizing twice compares equal.

**A constant dimension is defined to be exactly `0.0` in every bucket**, with
`DimensionStats::constant` set. `0/0` has no non-arbitrary answer; this one is deterministic, never
produces `NaN` or infinity, and makes the column inert for any downstream distance measure, which
is correct for a column carrying no variation. The flag is what stops a reader mistaking it for a
column of genuine zeros.

**Median/MAD was considered and is the wrong choice here**, which is worth saying because the
reverse is usually true. These are counts in short buckets, so most buckets are zero; when more
than half a series is zero the median is zero and the absolute deviations *are* the series, so the
MAD is zero too. Median/MAD degenerates on essentially every dimension at these widths, where
mean/stddev degenerates only on genuinely constant ones. Robustness is also not what this substrate
wants: a burst is an outlier by construction and it is the signal, not contamination. The sparsity
is measured by a test rather than assumed, so a fixture that stopped being sparse would force the
argument to be made again. The crossover is real and recorded — widen to 30 s on a real session and
the majority of buckets become occupied, at which point the argument stops applying.

Only one policy is implemented.

### 4. The synthetic oracle

`fixtures/synthetic-behavioral-oracle.ndjson`, 196 records, 240 s of span, schema v2, generated by
`witnessglass::experiment::oracle` and regenerable byte for byte with
`cargo run --example behavioral-signal -- --emit-oracle`. A test asserts the committed bytes equal
the generator's output, so the fixture cannot drift from the structure it claims to hold — the
declared structure is in constants, the recording is generated from them, and the tests assert
against the constants.

```text
      0 ..  60000   baseline    one two-record call every 6 s, one tool name
  60000 ..  90000   motif       a four-call, nine-record figure every 8 s, exactly
  90000 .. 150000   baseline    identical in shape to the first
 150000             regime change
 150000 .. 210000   elevated    a two-record call every 1.5 s, two tool names,
                                ~9x larger recorded responses, one subagent pair,
                                and the reported channel silent throughout
 210000 .. 240000   recurrence  the same figure with deterministic noise: jittered
                                starts and offsets, and one call that fails
 240000             session ends
```

The regime change is deliberately multivariate — rate, tool mix, recorded response size, and
channel occupancy all move at 150 s — so a projection that recovers only one of the four has lost
something and the test says so.

Built only from the v2 vocabulary as it actually is. No field is invented or stretched; in
particular `parent_agent_id` is absent on both subagent records, because the integration this
imitates does not supply it and a fixture that invented parentage would teach the substrate
something untrue. Every line contains `synthetic`; there is no real path, host, repository, or
command anywhere in it; a test asserts both per line. Because it is generated, "obviously
synthetic" is a property of the generator rather than of whoever last hand-edited a file.

Noise is a fixed LCG seeded from the interval. Reproducible noise, which is the only kind an oracle
can have.

### 5. Validation

`scripts/check.sh` — shell syntax, `cargo fmt --check`, clippy with `-D warnings`, the full suite,
and `scarp doctor`. Passing before any change was made (baseline) and passing after. No existing
test was modified. **24 new tests**, none of which runs a detector:

- fixture regenerable byte for byte; synthetic per line; replays as a complete v2 recording with
  no anomalies
- buckets contiguous, evenly spaced, inclusive of the last record; every row full width
- receipts reconcile to exactly `1..=196`, ascending within each bucket, agreeing with the
  `records` column
- empty buckets present with zeros, and a majority of buckets empty
- width parameterised: 500/1000/8000/240000 ms give 481/241/31/2 buckets with conservation at each
- dimension identity and order asserted as an exact list
- tool names verbatim; no category column, asserted against a deliberately tempting recording
- v1 and v2 kinds never merge
- `distinct_correlation_ids` counts ids present, not calls
- motif repeats **bit-identically** at its declared 8 s period across all four instances
- regime change visible on four dimensions at once
- recurrence carries the same record count, exactly one injected failure, and is *not*
  bit-identical to the original
- normalization pure, non-destructive, finite; zero mean and unit variance on every varying
  dimension; constant dimensions exactly zero
- sparsity measured, as the standing evidence for the policy
- no records → no axis → no signal, rather than an empty matrix over an invented axis
- truncated recording projects its valid prefix, carries `ValidPrefix` scope, and does not extend
  its axis to the session boundary the complete recording had
- span shorter than one bucket → one bucket; partial final bucket reported and never scaled
- backwards timestamps counted with receipts and **not repaired** — the record lands in the bucket
  its own timestamp names, beside a record it does not follow in append order

Salient output, against the oracle:

```text
scope: complete recording, 196 record(s)
axis: recorded_at — descriptive metadata, NOT the canonical order
  span_ms: 240000   bucket_ms: 500   samples: 481
  final bucket: full width, observed to 0 ms into it; nothing is scaled or extrapolated
  non-monotonic records: 0
  receipts across all buckets: 196 (every record placed exactly once, by its own timestamp)

  idx  dimension                                 sum      mean    stddev     max   nonzero constant
    0  records                                   196    0.4075    0.8056       3       105       no
    1  channel:reported                            8    0.0166    0.1279       1         8       no
    2  channel:observed                          186    0.3867    0.7685       3       103       no
    3  channel:recorder                            2    0.0042    0.0643       1         2       no
   10  kind:v2:tool_denied                         0    0.0000    0.0000       0         0      yes
   13  tool_name:SyntheticReader                  56    0.1164    0.4548       2        31       no
   17  distinct_correlation_ids                  111    0.2308    0.4591       2       103       no
   18  recorded_response_json_bytes            18875   39.2412  113.4503     413        91       no

  t=   60.000s  n=3   [3 1 2 0 0 0 1 1 1 0 0 0 0 2 0 0 0 1 45]     <- motif instance 1
  t=   68.000s  n=3   [3 1 2 0 0 0 1 1 1 0 0 0 0 2 0 0 0 1 45]     <- instance 2, identical
  t=  149.500s  n=0   [0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0]
  t=  150.000s  n=2   [2 0 2 0 0 0 0 1 1 0 0 0 0 0 0 0 2 1 413]    <- regime change
  t=  151.500s  n=2   [2 0 2 0 0 0 0 1 1 0 0 0 0 0 0 2 0 1 413]
  t=  210.000s  n=3   [3 1 2 0 0 0 1 1 1 0 0 0 0 2 0 0 0 1 45]     <- recurrence begins
  t=  210.500s  n=1   [1 0 1 0 0 0 0 1 0 0 0 0 0 0 1 0 0 1 0]      <- and the jitter shows
```

The motif's four instances are byte-identical runs of buckets. That is the property a Matrix
Profile would key on, asserted here without running one.

### 6. Desire-path friction

**Two collections' section templates were only discoverable by being refused.** `scarp new sprint`
rejected a `## Hypothesis` heading, naming the four sections a sprint has. The error is excellent —
it names the offending line and lists the valid sections — and the workflow that produced the bad
body was the reasonable one: read an existing artifact first. `sprint.md` for sprint:3 has a
`## Outcome` section, so the obvious inference from reading a real sprint is that `Outcome` is a
section a body may fill, and it is not; it is added at close. **Reading an existing artifact is
actively misleading about the template**, because a closed artifact carries sections a new one
cannot have. idea:2 covers exposing a template before the first artifact exists; this is the
adjacent case where artifacts exist and reading one still gives the wrong answer. The workaround
was one failed command and a re-edit, which is cheap; the smallest useful affordance is the same
one idea:2 asks for, and it would help here too.

**Appending a Result and an Outcome is still `cat >>`**, for the fourth round running. `scarp close`
takes a reference and nothing else, and neither a task nor a sprint is created with a `## Result` or
`## Outcome` placeholder, so both were appended by hand with the heading level matched against a
template nothing validates. Already idea:1, extended by idea:4; nothing new to add beyond another
occurrence, which is the point of counting them.

**Everything else was frictionless.** `scarp new sprint --body-file` and `scarp new task --sprint
sprint:4 --body-file` did exactly the right thing once the sections were right, and `scarp doctor`
runs inside the same gate as everything else.

### 7. What the real event schema turned out to be like

Run locally against an untracked 234-record real session. Not committed, not copied into
archaeology, and nothing in this task depends on its presence.

- **`channel:recorder` is constant zero on a real recording.** All 234 records arrive on `reported`
  or `observed`; the Claude adapter files session boundaries as *observed*, since a hook witnesses
  them, and the recorder asserts nothing. v2 permits both, so this is correct — but it means a
  three-channel dimension family has a structurally dead column for every hook-captured recording,
  and the substrate reports it as `constant` rather than dropping it. That was not anticipated.
- **A real recording is far emptier than the oracle.** At 500 ms: 2108 buckets, records in 119 of
  them — **94% empty**, against the oracle's 78%. The oracle is roughly four times denser than
  reality, which is an honest defect in an oracle built to be legible. Occupancy against width on
  that session: 500 ms → 5.6%, 1 s → 10.8%, 2 s → 19.9%, 5 s → 36.5%, 10 s → 48.1%, 30 s → 77.8%.
- **The reported channel is not a rounding error.** 65 reported-intent records against 82 tool
  requests. The channel distinction is a substantial fraction of the signal, not a decoration on
  it, which makes keeping the two as separate never-summed columns load-bearing rather than
  ceremonial.
- **`recorded_response_json_bytes` is violently heavy-tailed.** Mean 161, standard deviation 1349,
  maximum 23936 — about 17.6 standard deviations above the mean, in a single bucket. After
  z-scoring, one bucket dominates that dimension entirely. This is the one dimension where the
  choice against a robust scale has a visible cost, and it was still the right choice for the
  others.
- **No non-monotonic records anywhere.** The `recorded_at`-versus-`sequence` disagreement the axis
  is built to survive did not occur in this recording. The handling is implemented and tested and
  **unexercised in real data**, which is the same distinction task:4 insisted on and is repeated
  here about this layer.
- **`duration_ms` is populated and unusable anyway.** It arrives on completions in this session,
  and it is still not a dimension, because "arrives on completions" is not "arrives on every
  record" and a matrix column has no way to say "not supplied".

### 8. Recommendation for the next single detector experiment

**Haar DWT.** Not Matrix Profile, and not changepoint detection, and the reason is the sparsity
measurement rather than a preference among algorithms.

The finding that should drive the next round is that **there is no single correct bucket width**.
At 500 ms a real session is 94% empty; at 30 s it is 22% empty and every sub-second structure the
substrate was built to resolve has been averaged away. Every other detector requires committing to
one width before it runs, and that commitment is currently unjustified — a Matrix Profile over a
94%-empty series would find that almost every subsequence matches almost every other subsequence,
because they are all mostly zeros, and it would be right and useless. Changepoint detection has the
same problem from the other end: with 5.6% occupancy, the dominant "change" available to find is
where the empty stretches are.

A Haar transform is the one of the three that *answers* the width question instead of presupposing
it. It is a dyadic multiscale decomposition — 500 ms, 1 s, 2 s, 4 s, and upward, all in one pass —
so its output is exactly the missing evidence: at which scale does this recording carry energy? It
is also the cheapest to implement honestly (a few dozen lines, no dependency, no parameter to tune,
and the boundary handling is the one thing to get right), and the easiest to falsify: the oracle's
8 s motif period and 60 s regime blocks should show as energy at identifiable scales, and if they
do not, the substrate is worse than it looks and the round ends there.

Then run Matrix Profile at whatever width the transform says carries structure. Doing it in the
other order means guessing.

### What this task did not do

No detector. No change to the raw format, the schema, the recorder, `inspection`, or the viewer. No
new dependency. No product CLI surface — `witnessglass --help` is unchanged and still lists four
verbs. No real recording committed, copied, or depended on. Nothing pushed.

Whether the hypothesis holds is still open, and this round could not settle it: it established
that the numbers handed to a detector are honest, regular, and traceable, not that a detector
recovers anything from them.
