---
id: spr_01KZ9R11YTDBKH39X3FDP68EZR
sequence: 8
kind: sprint
status: closed
created: 2026-08-05
closed: 2026-08-05
---

# Figure

## Goal

Find out whether the representation, rather than the detector, is what sprint:6 ran into.

sprint:6 ran a univariate Matrix Profile over sprint:4's 500 ms raster at six windows, on two
fixtures and one real recording, and its central finding was not about windows at all:

> Uniformly sampled, univariate sparse signals make trivial lone-event coincidences appear to be
> perfect motifs. The detector did not reliably recover the deliberately planted recurring figure.

Every top masked motif in every sparse dimension was a pair of windows holding **one** non-empty
bucket at the same relative offset. That is a perfect match by construction and says nothing about
behaviour.

This sprint tests the smallest plausible alternative:

> Can a simple event-native sequence representation — events kept as events, in order, with their
> relative timing, compared directly — recover the deliberately planted recurring figure that
> sampled univariate Matrix Profile failed to recover?

And, secondarily:

> How much timing and event variation can that representation tolerate before genuinely related
> occurrences stop matching?

## Rationale

sprint:6 recommended exactly this experiment and named why: the limiting factor was not the window
parameter it set out to test. It is that fixed-width sampling of a 94%-empty event stream produces
subsequences whose z-normalized shape is dominated by *where a single event sits inside the window*
rather than by what happened. Rasterizing an event stream into mostly-empty buckets throws away the
two things a recording actually has — which events, in which order — and keeps the one thing it is
worst at supplying, a dense regular amplitude.

**This round earns or rejects one prerequisite, and builds nothing on top of it.** The larger
corpus-motif architecture that would sit downstream — cross-recording accumulation, motif families,
variable-length discovery, path or payload facets — is not started here and would each need its own
argument. If a fixed-event-count matcher over event identity and relative timing cannot recover one
figure a fixture was built to contain, none of that is worth attempting.

**Falsifiable in one round.** Either the injected figure is recovered with separation from a
deliberately destroyed control, or sparse behavioural motif detection is not worth further effort at
this representation either. Both outcomes are publishable results about the representation and
neither is a verdict on any particular library.

### What the representation is allowed to know, and what it is not

The primary metric may use **event identity and relative timing, and nothing else**. Identity means
the schema-tagged event kind the record carries and the tool-name string the integration delivered,
byte for byte. It does not mean a category.

If the recording knows `tool_name:SyntheticReader`, the representation may know
`tool_name:SyntheticReader`. It may not promote that to `filesystem_read`, `inspection`, or
`research`. `CLAUDE.md` §2 forbids exactly that promotion and sprint:4 already refused it for the
sampled substrate; the refusal does not weaken because a new representation would find it
convenient.

Deliberately deferred, and *not* because they would be illegal — several are mechanically derivable
from raw evidence and would be legitimate facets for a later round: delivered paths, path regions,
file extensions, recorded input and response JSON byte counts, edit payload sizes, working-set
unions. They are excluded here so the first question stays the first question. Anything of this kind
encountered while implementing is inventoried in the task Result as a future candidate facet, with
its provenance stated precisely, and used by nothing.

Reported intent is not a facet either. It is a channel, and this round's primary run excludes it —
see the task's preregistration for why the harder scope is the primary one.

## Success criteria

- An event-native representation that preserves event identity, order, and relative timing, derived
  from the same validated projection everything else in this project reads, and rebuildable from raw.
- A sequence distance small enough to hand-check, with an inspectable decomposition: event edits and
  timing reported separately from the combined figure, never hidden inside one scalar.
- A timing policy that tolerates the fixtures' deliberate jitter without treating a 200 ms
  discrepancy at 100 s as it treats one at 1 s, chosen and documented before any fixture is run.
- Known-answer microtests over sequences small enough to check by eye, passing before the oracle is
  touched.
- A preregistered event-count ladder derived from the committed fixtures' own constants, including
  one window shorter than the planted figure, the figure's exact length, and longer ones.
- Both synthetic fixtures run at that ladder, with an explicit and documented trivial-match exclusion
  policy, and results reported per window with locations, distance components, and whether a pair
  corresponds to known planted structure.
- Deterministic nulls that destroy event order and timing separately, run through the identical path,
  with the comparison metric fixed in advance.
- A controlled perturbation sweep, run only if basic recovery is earned, showing whether the metric
  degrades gracefully as a figure becomes less like itself.
- A direct three-way comparison with sprint:6's sampled Matrix Profile result: supported, mixed, or
  falsified, kept separate from any opinion about the metric's elegance.
- `scripts/check.sh` passes unweakened. No existing test changed.

## Non-goals

- Cross-recording or corpus motif accumulation, motif-family clustering, variable-length motif
  discovery, MinHash, or Jaccard path-set similarity.
- Path, extension, repository-region, payload-magnitude, edit-delta, or reported-intent *features*
  in the primary metric.
- Semantic tool categorization, in any form, for any reason.
- Learned embeddings, neural representations, LLM similarity, or any opaque high-dimensional feature
  vector. The point is to test the representation, not to outsource the question.
- A generic `MotifDetector` framework, a public stable motif schema, or an ecosystem around a
  sequence primitive. The implementation should be cheap to delete.
- Cantrip generation, Scarp proposal generation, A/B testing, historical replay, or workflow
  compilation.
- Expanding the Behavioral Spectroscope. sprint:7 completed it; if this round's output would be
  illuminating visually, the smallest useful visualization is *recorded* as a follow-up and not
  implemented.
- Any change to the raw format, the schema, the recorder, `inspection`, the viewer, or the product
  CLI. Any weakening of an existing check. Any real recording committed or copied.

## Outcome

One task, closed. **Supported.** Keeping events as events reverses sprint:6's central failure.

sprint:6's finding was that a sampled univariate Matrix Profile over a 78–94% empty raster ranked
coincidences of two lone events above the figure the fixture was built to contain, at every window,
on both fixtures, in every dimension. This sprint replaced the representation and kept everything
else — the same fixtures, the same discipline, a null of the same shape, a preregistration committed
before the matcher existed — and the planted figure comes back at every rung of both ladders.

The single number worth carrying forward is unanchored: with degenerate windows excluded, **the
global minimum over every disjoint pair of windows in the whole recording is a pair of planted
occurrences, at distance zero**, on both fixtures and in both channel scopes. Nobody supplied a
region. The detector was asked what the most similar non-degenerate pair anywhere was, and it
answered with the figure.

### Success criteria, against evidence

- **An event-native representation.** `MarkedEvent` — a schema-tagged kind, a verbatim delivered tool
  name, an offset, and the gap from the previous retained event. Built from `Inspection` like every
  other projection here, borrowing rather than owning, reading no file and consulting no clock.
- **A hand-checkable distance with an inspectable decomposition.** Substitution 1.0, indel 1.0,
  timing 0.5 × a bounded log-ratio. Ten reported quantities per comparison, three of them distances
  that are never collapsed into one.
- **A timing policy fixed in advance.** 1.0 s against 1.2 s is 0.12; 100 s against 100.2 s is under
  0.005; the same 200 ms between 0.1 s and 0.3 s is 0.50. Asserted against the four values task:18
  preregistered.
- **Microtests before the oracle.** Twelve tests passing before any fixture scan ran, including one
  that proves the perturbation sweep's hand-built base figure is the committed fixture's own.
- **A ladder derived from the fixtures' constants**, containing a short control, the exact figure
  length, and two longer rungs, for each of four (fixture, scope) combinations.
- **Both fixtures, with an explicit exclusion policy.** Two windows are compared only when they share
  no event — stricter than the `ceil(m/4)` zone sprint:6 inherited, and stricter on purpose.
- **Two deterministic nulls**, one destroying order and one destroying timing, run through the
  identical path. The order null separates by `+0.13` to `+0.49`. The timing null barely separates,
  which was predicted in advance and is a finding about where the discrimination lives.
- **A perturbation sweep, earned and run.** The distance degrades gracefully: 0.000 exact, 0.016 at
  10% jitter, 0.051 at 30%, 0.077 to 0.105 for one structural change, 0.725 for an unrelated figure.
- **A three-way verdict, kept separate from the implementation's merits.** §13 of the task Result.
- **`scripts/check.sh` passes unweakened**, no existing test changed, no dependency added, and no
  `Cargo.toml` change at all.

### What the sprint found that it was not looking for

**A three-event window already recovers the figure, and the null is what says it should not be
trusted.** The short control was predicted to fail and does not — the figure's first three observed
events carry three distinct marks where a baseline fragment carries two. But its null separation is a
third of the figure length's, and the global order null reaches zero there. Recovery and
discrimination are different questions and the short rung answers only the first.

**This representation has its own abundant trivial match, and it is not the same one.** Exactly
periodic repetitions of a *degenerate* two-mark figure fill the unrestricted global ranking on both
fixtures. Unlike sprint:6's lone-event coincidences they are genuine multi-event repetitions — real,
and uninteresting. One column separates them, and that column is a heuristic that fits these
fixtures rather than a principle: a real recurring figure built out of one tool name would be
indistinguishable from an idle loop built out of one tool name.

**A preregistered ranking criterion was mis-specified for the second consecutive round.** S2 asked
for a cross-region pair in the query's top five on a fixture whose region A holds thirty exactly
periodic occurrences, twenty-nine of which precede it at distance zero. A criterion no perfect
detector can satisfy. Kept as written, reported beside the measurement it should have taken, and
named as a pattern rather than an accident.

**The real recording produced a candidate a human agrees with.** sprint:6's strongest real-session
motif was two lone spikes ten minutes apart, of which that round wrote that a human reading the
projection does *not* agree they are similar. This round's is eight events carrying the same four
marks in the same order, differing only in spacing. No ground truth, no tuning, and the same
recording.

### What this sprint deliberately leaves open

The one recommended next step is cross-recording figure matching on two real recordings — the
smallest experiment that could invalidate the whole direction, and the prerequisite every downstream
hypothesis needs. Nothing here started it.

Everything in §14 of task:18's Result: the deferred similarity facets, each inventoried with the
claim it would actually be making. Variable-length figures. sprint:5's changepoint recommendation,
untaken and untouched. And the visualization this result would suit, recorded as a suggestion and not
implemented — the Behavioral Spectroscope is a completed experiment and this sprint did not reopen it.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, the
workbench, the Spectroscope, or the product CLI's verbs, and no dependency was added.
