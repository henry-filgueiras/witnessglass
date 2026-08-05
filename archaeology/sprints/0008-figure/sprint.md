---
id: spr_01KZ9R11YTDBKH39X3FDP68EZR
sequence: 8
kind: sprint
status: active
created: 2026-08-05
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
