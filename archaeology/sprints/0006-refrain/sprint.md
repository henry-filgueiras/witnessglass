---
id: spr_01KZ7A9ZHPYSQH7WATWCZXA133
sequence: 6
kind: sprint
status: closed
created: 2026-08-04
closed: 2026-08-04
---

# Refrain

## Goal

Find out whether the multiresolution evidence sprint:5 produced can actually constrain Matrix
Profile's window parameter — not whether a Matrix Profile can be made to run.

Two questions, and the second is the one that can embarrass us:

> Can Haar-derived scale evidence reduce Matrix Profile's window-selection arbitrariness enough to
> recover known repeated behaviour in sparse synthetic WitnessGlass signals and identify plausible
> recurring or discordant regions in a real recording?

> Does choosing windows near Haar-indicated scales actually outperform windows that Haar evidence
> would make us less likely to choose?

## Rationale

sprint:5 recommended changepoint detection and this sprint does something else, on instruction and
for a reason worth writing down: the Haar round produced a *parameter-selection story* — coarse
scales carry the structure, fine scales are indistinguishable from sparsity — and a story of that
shape is exactly the kind this project has twice been wrong about. sprint:3 exists because a chain
of individually reasonable steps produced a false conclusion inside one system asking itself whether
it was complete. A satisfying multiresolution narrative used to pick a Matrix Profile window is the
same shape of risk, and the only way to find out is to run the control.

**Haar scale and Matrix Profile window are not the same quantity, and sprint:5's language did not
always keep them apart.** A Haar level is a *contrast between adjacent means*, and its diagnostic
for a periodic train is a cliff at the level whose half-window reaches the period. A Matrix Profile
window is the *length of the pattern being matched*. The injected motif has three separate numbers
attached to it — an instance duration of about 1–2 s, a repetition period of 8 s, and a Haar cliff
at the 16 s level — and there is no reason to expect a good Matrix Profile window to coincide with
any particular one of them. This sprint has to say which it is testing at every point.

There is also a specific way this could all be moot, and it is predictable in advance rather than
after. These signals are 78–94% empty. `motif-rs` documents its convention for constant
subsequences — two constant subsequences are at distance exactly 0 — so in a mostly-empty series
the global minimum of the matrix profile is 0, attained by two stretches of nothing. Any result
read off an unmasked profile would be a statement about emptiness. That is a representation problem
rather than a bug, and if it dominates, the finding is that sampled univariate Matrix Profile is the
wrong shape for this data.

## Success criteria

- An implementation decision made on inspected evidence — license, maturity, dependency footprint,
  validation against a reference — rather than on convenience, and recorded with what was
  inspected.
- The library's behaviour on exclusion zones, constant subsequences, normalization, and
  index-to-time conversion pinned by tests, so a later reader is protected from *our*
  misunderstanding of it as much as from its defects.
- A window ladder of four to six lengths, derived from committed evidence and written down with its
  predictions **before the detector is run on any fixture**, including at least one control window
  drawn from the region sprint:5 found indistinguishable from an impulse null.
- A null that answers "what does a good motif distance look like when temporal order is destroyed",
  and a comparison metric fixed in advance rather than chosen once the numbers are visible.
- Per-dimension independence, as in both prior rounds. No multivariate fusion.
- An honest three-way verdict on the guidance hypothesis — supported, mixed, or falsified — kept
  separate from any verdict on Matrix Profile itself.
- Any representation failure recorded as a finding rather than patched around.

## Non-goals

- **Changepoint detection.** sprint:5 recommended it; it is still unimplemented and this sprint does
  not touch it.
- Multivariate or multidimensional Matrix Profile. The library ships `mstump` and `mmotifs`; this
  sprint uses neither. If independent dimensions turn out to lose essential structure, that is a
  finding and a later task's authorization, not something to reach for mid-round.
- Implementing STOMP, STAMP, STAMPI, or SCRIMP by hand if an adequate implementation exists.
- A detector trait, a plugin system, a registry, or any production analytics surface.
- Any change to the raw format, the schema, the recorder, `inspection`, or the viewer; any product
  CLI surface; any web UI work; any Python runtime or repository dependency.
- Committing, copying, or depending on a real recording.
- Rewriting sprint:5's conclusions. Its result stands as written, including the recommendation this
  sprint declines to follow yet.

## Outcome

One task, closed. The sprint's two questions have different answers, and the second one — the
control question, the one designed to embarrass a satisfying story — is the more useful.

**Can Haar-derived scale evidence reduce Matrix Profile's window arbitrariness?** Partly. It
correctly ruled out the bottom of the ladder: sprint:5 said levels 1–5 (≤16 s) of the real recording
were indistinguishable from an impulse null, and Matrix Profile finds separation of identically zero
at 2 s, 8 s, and 16 s. That was predicted before the detector ran, and it held.

**Does choosing windows near Haar-indicated scales actually outperform windows Haar would make us
less likely to choose?** Within the endorsed range, no. 32 s — the first Haar-informed window —
produced nothing. 128 s, where Haar's excess was *strongest*, produced an order of magnitude less
separation than 64 s. Haar's ranking of the coarse levels is not Matrix Profile's ranking. The
verdict is **Mixed**: a neighbourhood, not a window, and independent tuning is still required.

Kept separate, as the sprint required: that is a verdict on the *composition*. Matrix Profile itself
found nothing on these signals that survives its own null except in aggregate dimensions at long
windows, and what it found there is clustering rather than recurrence.

### Success criteria, against evidence

- **An implementation decision on inspected evidence.** `motif-rs` 0.1.0, MIT, ~11,200 lines, source
  read before adoption, STUMPY-validated at MAD `2.7e-12`–`1.2e-11` by its own comparison report,
  STUMPY-matching exclusion zone and sigma threshold, and — the thing that decided it — a
  *documented* constant-subsequence convention. Recorded with what would have disqualified it.
  Placed behind a non-default feature so a default build of the recorder links none of it.

- **The library's behaviour pinned by tests.** Exclusion zone, trivial matches, both
  constant-subsequence cases, offset and amplitude invariance, and index-to-time conversion, each
  against a vector small enough to check by hand. These protect against *our* misunderstanding as
  much as against the library.

- **A ladder and predictions written before the run.** Six windows, four predictions, a comparison
  metric, and a falsification threshold, committed at `363ac20` before the detector met a fixture.
  The ladder did not move afterwards — including when the round found a defect in its own input
  handling mid-run.

- **A null that answers the right question.** A fixed-seed shuffle preserving the value multiset and
  destroying temporal order. Its answer is uncomfortable and load-bearing: **the null reaches
  distance 0 too**, at every window up to 32 s, on both fixtures. A perfect match in this
  representation is the default state, not a discovery.

- **Per-dimension independence.** No fusion; `mstump` and `mmotifs` ship in the library and went
  unused.

- **A three-way verdict, kept separate from any verdict on the detector.** Above.

- **Representation failures recorded rather than patched.** Seven, ranked by how much each decided
  the round.

### What the sprint found that it was not looking for

**Sampled univariate Matrix Profile matches lone events, not figures.** Two windows each holding one
non-empty bucket at the same relative offset are identical after z-normalization and score exactly 0
regardless of context. In a 78–94% empty signal those pairs are everywhere, and they occupy the top
of every masked motif list at every window in every sparse dimension of both fixtures and the real
recording. Even the best synthetic result in the round — the cross-region recurrence recovered at
rank 1 with separation `+0.384` — is two 256-sample windows each containing **one** record. The
detector never matched the injected figure, on a fixture built to contain one, at a window where it
identified the correct pair of regions.

This is why the recommendation is an event-native motif method rather than another window, another
wavelet, or a multivariate profile: the limiting factor is not the parameter this sprint set out to
test.

**Stacking a global normalization in front of an internally-normalizing detector manufactures
motifs.** The first pass fed sprint:4's z-scored column on the reasoning that the metric is
scale-invariant, which is true in exact arithmetic and false in floating point. An all-empty window's
rolling standard deviation came out at `1.863e-9` instead of `0`, defeating the constant test on 275
of 309 empty windows, after which z-normalization amplified pure rounding error into a
full-amplitude shape and the detector reported a flawless motif between two regions holding no
records. Found by disbelieving an obviously wrong intermediate rather than by a test. Fixed by
passing raw counts, which changes no distance in exact arithmetic and is not a change to sprint:4's
policy. Both the requirement and the hazard are now tests.

**Discords are worth more than motifs here, and the reason is structural.** In a 94%-empty signal
"most unlike everything else" is well posed because dense regions are rare, while "most similar to
something else" degenerates because empty regions are everywhere. The strongest real motif candidate
was two lone tool completions ten minutes apart that a human reading the projection does not accept
as similar; the strongest discord was the recording's densest sustained stretch, which a human does
accept as unusual. Both readings are recorded as experimental interpretation, not as evidence.

**The substrate's refusal to classify tool names has a measurable cost.** `tool_name:Read` on the
real session is 81–99% constant across the entire ladder — too sparse to support a Matrix Profile at
any window. Aggregating tool names would fix it and remains unauthorized. task:14 accepted that
trade on principle; this sprint priced it.

### What this sprint deliberately leaves open

Changepoint detection, still. sprint:5 recommended it, this sprint did not implement it, and nothing
here weakens the case — the separation results that do exist measure clustering, which is more
support for it rather than less. It remains the right second step, behind the representation
question.

Whether an event-native representation recovers what a sampled one could not. That is the
recommended next experiment and it is falsifiable in one round.

Whether multivariate methods would help. Not investigated on purpose: the evidence says univariate
*sparsity* is the problem, and a multidimensional profile over the same representation would inherit
it. If the event-native round says otherwise, that authorization can be sought then.

sprint:4's normalization policy stands, untouched, and is now known to be the wrong input for this
class of detector for numerical rather than statistical reasons.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, or the
product CLI. One optional dependency was added, off by default.
