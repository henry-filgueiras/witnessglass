---
id: spr_01KZCCTGTQM5J9CXX46A4R53C1
sequence: 20
kind: sprint
status: active
created: 2026-08-06
---

# Doublet

## Goal

Rerun sprint:19's exact search-aware calibration against a null that preserves each recording's
first-order categorical transition structure instead of treating event order as exchangeable, and
determine whether the separation sprint:19 measured survives.

## Rationale

sprint:19 reached verdict A and immediately bounded it. Every observed recording has an immediate
repetition rate of exactly zero and a mean run length of exactly one; the order null, which shuffles
marks, produces repeats constantly. The four specimens sit outside the entire null range on the most
trivial local statistic there is. So the separation supports one claim only — the search detects
sequential structure not explained by exchangeable ordering — and that claim is consistent with the
schema's own request→outcome coupling and nothing more.

sprint:19 named the narrowest repair itself: resample each sequence from its own first-order
transition structure, so vocabulary, marginals, length and the alternation all survive while
longer-range reuse does not. This round runs that, changing nothing else — same `T`, same complete
search, same `B`, same threshold, same specimen and `k` grid — so the two rounds are paired and the
mechanical change is readable specimen by specimen.

The falsifiable prediction is sprint:19's own explanation of itself: if the separation was mostly
first-order grammar, preserving that grammar should substantially reduce it. A collapse is the more
valuable outcome, and this round is built so that a collapse is reportable rather than avoidable.

There is a construction choice here that decides what may honestly be claimed, and it is settled on
measured nuisance fidelity before any criterion about `T` is written: ordinary Markov resampling
preserves transition counts only in expectation, and at 32-event recordings "in expectation" is not
"preserved".

## Success criteria

- sprint:19's premises re-verified from the repository, and its numbers reproduced before anything new
  is built. Any discrepancy recorded.
- The first-order null specified exactly — state space, estimator, initial state, unseen transitions,
  dead ends, length, seeds — with exact versus in-expectation preservation stated property by property
  and asserted by tests rather than claimed in prose.
- Null adequacy decided before motif calibration, on the same summaries sprint:19 used to condemn the
  order null, plus transition-fidelity measurements. An inadequate null stops the round.
- Controlled fixtures whose background is itself first-order, and a planted figure whose recovery is
  not available from first-order counts. Contamination measured, not assumed away.
- The identical `complete_search` on observed and null paths, every data-dependent stage rerun inside
  every replicate, asserted by a test.
- A paired sprint:19-versus-sprint:20 table over the same specimen and `k` grid, with the mechanical
  change stated in preregistered quantities.
- A propagation pass over every mechanism the construction and adequacy work discovers, checked on
  every branch including the ones that cannot fail.
- Specimen-level verdicts; the corpus never forced to one.

## Non-goals

No change to R1, the complete search, candidate generation, ranking, deduplication, boundary
constraints, top-k reporting or real-corpus hygiene. No second-order model, richer marks, semantic
categories, timing features or paths. No new statistic, no new detector, no aggregate invented to tidy
a result. No adoption of R1 whatever the outcome. No threshold chosen after seeing data. No
interpretation, naming or inspection of any discovered span. No new specimen admitted to decision:8's
inventory. No recording content in any artifact. Nothing pushed.
