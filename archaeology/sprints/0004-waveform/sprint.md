---
id: spr_01KZ75RKADD2ENBZR202PFZ1RF
sequence: 4
kind: sprint
status: closed
created: 2026-08-04
closed: 2026-08-04
---

# Waveform

## Goal

Find out whether a validated WitnessGlass recording can be projected into a regular, normalized,
multivariate numerical signal that preserves enough observable temporal structure for ordinary
numerical algorithms to work on — and to find that out cheaply enough that a negative answer costs
one `git rm`.

This sprint builds the substrate and nothing that consumes it. No detector.

## Rationale

Three sprints of evidence work have produced a recording whose semantics are settled, a projection
that carries receipts for every derived claim, and a viewer that renders one recording for a human
to read. Every question this project can currently ask of a recording is answered by a person
looking at it.

There is a different class of question — does this session contain repeated structure, does its
behaviour change regime partway through, is there activity at more than one timescale — that a
person reading a ledger is bad at and that numerical methods are good at. Matrix Profile,
changepoint detection, and a Haar wavelet transform all want the same input: a regular, evenly
spaced, normalized multivariate series. WitnessGlass does not produce one. It produces irregular
events on an append chain whose canonical order is a sequence number and whose only clock is
descriptive metadata that explicitly does not establish order.

The gap between those two things is the whole risk, and it is an epistemic risk rather than a
numerical one. Bucketing events into fixed-width time bins requires using `recorded_at` as an axis,
which the project has said three times over is not what `recorded_at` is for. Giving those bins
useful dimensions invites exactly the promotion decision:2 forbids: a tool named `Bash` is not
observed evidence of "shell activity", a tool named `Grep` is not observed evidence of "search", and
a `file_path` key inside an uninterpreted input blob is not observed evidence that a file was
touched. A substrate that quietly makes any of those three moves would produce beautiful signals
about a session that did not happen.

So the experiment is worth running and worth fencing. It is run as an experiment, in a module that
is marked disposable in its own documentation, behind an example binary rather than the product CLI,
and it is deleted rather than defended if the answer is no.

### The hypothesis under test

> A validated WitnessGlass event stream can be projected into a regular, normalized multivariate
> behavioral signal that preserves enough observable temporal structure for numerical algorithms to
> recover deliberately injected motifs, regime changes, or multiscale behavior.

Falsifiable in this sprint only up to the substrate: a deterministic synthetic recording with known
hidden structure either survives the projection with that structure intact and measurable, or it
does not. Whether an algorithm then *finds* the structure is the next sprint's question, and this
sprint deliberately does not answer it.

## Success criteria

- A derived projection turns an already-validated `Inspection` into a contiguous, evenly spaced,
  multivariate series with no gaps, and it is rebuildable from raw with nothing cached or written.
- Every dimension is licensed by evidence that is actually in a record. A dimension that requires
  classifying a delivered string into a category, or reading meaning out of an uninterpreted input
  payload, is not built, and the refusal is written down where the substrate is defined.
- The time axis is built from `recorded_at` and says so, everywhere it is exposed, including what
  that costs: it is descriptive metadata, it is not the canonical order, and a recording whose
  timestamps move backwards produces a signal whose bins disagree with its append chain. Nothing
  reorders raw evidence to make the axis tidier.
- Empty bins, the partial final bin, sparse dimensions, zero-variance dimensions, and a truncated
  recording each have defined, tested behaviour, and none of them is filled in with a plausible
  value.
- A deterministic synthetic oracle recording exists whose hidden structure is known in advance and
  is regenerable byte-for-byte from committed code.
- The normalization policy is chosen against the measured shape of the data rather than by default,
  documented, and does not destroy or replace the unnormalized counts.
- The substrate is deletable: one module, one example, one fixture, one test file, and nothing in
  the product CLI, the schema, the recorder, the projection, or the viewer depends on any of it.

## Non-goals

- **Any detector.** No Matrix Profile, no changepoint detection, no wavelet transform. Enough
  arithmetic to validate the substrate, and not one function more.
- Any change to the raw recording format, the schema, the recorder, `inspection`, or the viewer.
  If the experiment wants a field the projection does not expose, the experiment goes without it
  and says so.
- A detector trait, a plugin system, a registry, a generalized analytics framework, or any surface
  that would make a second detector cheaper at the cost of committing to a first.
- A public `analyze` command. The product CLI keeps its four verbs.
- Any numerical, plotting, dataframe, or Python dependency.
- Deciding what a signal *means*. This sprint produces numbers about observed records. It infers no
  intent, no productivity, no phase of work, and no explanation.

## Outcome

One task, closed. The substrate exists, it is validated against an oracle whose structure was
decided before the projection ran, and it is deletable in one commit. The hypothesis this sprint
was commissioned to test is **not settled**, and the sprint said in advance that it would not be:
it is falsifiable here only up to the substrate, and the substrate survived.

### Success criteria, against evidence

- **A derived projection produces a contiguous, evenly spaced multivariate series, rebuildable
  from raw with nothing cached.** `witnessglass::experiment::signal::project` is a pure function
  of `(Inspection, BucketWidth)` that reads no file, consults no clock, and borrows throughout.
  Two projections of one replay compare equal and the replay is unchanged by either — asserted,
  not asserted about.

- **Every dimension is licensed by evidence in a record, and the refusals are written down.** Six
  families, each a count or a measurement of something literally present. The refusals — semantic
  tool categories, command-content classification, files touched, output volume, `duration_ms`,
  `prompt_id` segmentation — are documented beside the definitions *and* asserted by a test built
  on a deliberately tempting recording carrying `Bash`, `Read`, `Grep`, a `cargo test` command,
  and a source path. It produces three verbatim columns and no category column.

- **The axis is `recorded_at`, says so everywhere, and carries what that costs.** `TimeAxis` exists
  as a type so a caller holding a matrix also holds the caveats. Nothing is reordered; a record is
  placed by its own timestamp and stays in canonical order otherwise; the non-monotonic count is
  carried through from `inspection` with its receipts and is explicitly not repaired. A test
  builds a recording whose clock moves backwards and asserts the disagreement is reported rather
  than resolved.

- **Boundaries defined and tested, none filled with a plausible value.** Empty bucket, first and
  last bucket, partial final bucket, sparse and zero-variance dimensions, truncated valid prefix,
  no records at all, a span shorter than one bucket, and backwards timestamps. The no-records case
  returns nothing rather than a zero-row matrix over an invented axis.

- **A deterministic oracle exists and is regenerable byte for byte.** 196 records, structure
  declared in constants and generated from them, with a test asserting the committed bytes equal
  the generator's output.

- **The normalization policy was chosen against measured data.** z-score, with median/MAD rejected
  on a sparsity measurement rather than on taste, the measurement itself held by a test, and the
  crossover width at which the argument stops applying recorded beside it.

- **Deletable.** One module, one example, one fixture, one test file, two lines in `lib.rs`. The
  product CLI is unchanged and still lists four verbs.

### What the sprint found that it was not looking for

**There is no single correct bucket width, and the default is wrong for whole-session work.** A
real 234-record session is 94% empty at 500 ms and 22% empty at 30 s. The substrate was built
expecting the width to be a detail; it is the central unresolved parameter, and it changed the
recommendation for the next round from Matrix Profile to Haar DWT — the one detector of the three
that answers the width question instead of presupposing it.

**A three-channel dimension family has a structurally dead column.** Every record of a real
hook-captured recording arrives on `reported` or `observed`; the adapter files session boundaries
as *observed*, because a hook witnesses them. `channel:recorder` is constant zero for that whole
class of recording. v2 permits both, so nothing is wrong — but the substrate reports it as
constant rather than dropping it, which is the correct behaviour arrived at for a reason nobody
anticipated.

**The reported channel is a substantial fraction of the signal**, not a decoration: 65 reported
records against 82 tool requests. Keeping reported and observed as two columns that are never
summed is therefore load-bearing rather than ceremonial.

### What this sprint deliberately leaves open

Whether any algorithm recovers any of the structure the substrate preserves. That is the next
sprint's question, and it needs a detector, which this sprint's non-goals forbid.

The next round is recommended as **one experiment: a Haar DWT**, on the reasoning in task:14 §8. If
the oracle's 8 s motif period and 60 s regime blocks do not appear as energy at identifiable
scales, the substrate is worse than it looks and the whole line of work ends there — which is what
"disposable" was supposed to mean when this sprint was commissioned.

Nothing here changed the raw format, the schema, the recorder, `inspection`, or the viewer, and no
new dependency was added.
