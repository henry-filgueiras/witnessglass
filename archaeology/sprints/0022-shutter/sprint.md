---
id: spr_01KZSCKFB7AFVM9XA9DA5HV6ZE
sequence: 22
kind: sprint
status: active
created: 2026-08-11
---

# Shutter

## Goal

Determine whether a fixed-budget Few Random Searches (FewRS) assay — 459 complete search-aware null
replicates per cell, certifying only when the observed statistic strictly exceeds every null statistic
— preserves enough of sprint:19's conclusion to justify its narrower evidence surface.

## Rationale

sprint:19 spent 999 complete searches per cell and read a Monte Carlo tail. sprint:20 spent the same
again under a different null. Both rounds' cost is dominated by one thing: how many times the complete
search has to be rerun inside a null replicate. FewRS claims a fixed, small budget derived from `alpha`
alone, with the complete analysis rerun inside every resample and no per-analysis multiplicity
correction. At `alpha = 0.01` that budget is 459 — a 2.18x reduction, not the 8-to-64-search figure the
paper's looser-alpha examples suggest.

This round is retrospective on purpose. The 999-replicate grid is frozen archaeology, the seed schedule
is deterministic, and the first 459 seeds are a prefix of the 999 already spent — so the assay can be
run and audited against a reference that already exists, without a fresh reference campaign and without
touching the null, the search, `T_k`, or the corpus.

The round is built so it can kill its own premise. A binary certification grid is a strictly narrower
instrument than a tail estimate: it cannot produce a median, a quantile, a percentile movement, or the
distributional statement sprint:20 rests on. If the cheaper instrument does not preserve sprint:19's
majority verdict, or if what it preserves is not worth the evidence it gives up, the useful output is a
small negative result and a recommendation to retire the idea.

## Success criteria

- Repository truth reconstructed from the code and archaeology first; every discrepancy between the
  commission and the repository recorded before any criterion is written.
- `m` derived from the FewRS formula rather than asserted, and the derivation pinned by a test.
- The decision rule strict: certification iff `observed > max(null)`. Ties do not certify.
- The unchanged `calibration::complete_search` rerun inside every null replicate, through the existing
  `calibrate` path, with no second implementation of the statistic and no change to the null generator.
- Synthetic controls executed before any observational cell, with each control rule's reachability
  checked — including the rules that cannot fail, and disclosed as such.
- All 30 sprint:19 `(pair, k)` order-null cells, none selected post hoc.
- The classification thresholds fixed before execution and not softened afterwards.
- Cost accounting in actual searches performed, not in the theoretical ratio alone.
- The boundary between binary certification and distributional calibration stated explicitly, with the
  quantities the binary assay provably cannot produce named.

## Non-goals

No general FewRS subsystem, no multi-analysis infrastructure, no adoption. No change to `T_k`, the
complete search, alignment ranking, deduplication, R1 semantics, the ladder, the null generator, the
seed schedule, or the specimen inventory. No normalizing or combining of statistics across `k`. No
claim of family-wise error control across the 30 cells. No claim that FewRS validates the existing LCG
or the doublet sampler. No fresh 999-replicate reference campaign. No recording content in any
artifact. Nothing pushed.
