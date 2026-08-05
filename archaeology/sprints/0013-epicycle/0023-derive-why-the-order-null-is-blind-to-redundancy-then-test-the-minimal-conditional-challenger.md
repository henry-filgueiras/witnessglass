---
id: tsk_01KZA0ZWNTWT60R828A01RSSJH
sequence: 23
kind: task
status: pending
sprint: spr_01KZA0ZWNEG3KMC464BN7CWNKD
created: 2026-08-05
---

# Derive why the order null is blind to redundancy, then test the minimal conditional challenger

## Objective

Determine whether sprint:12's Family E failure is a limitation of the global order-permutation null or
evidence that null-relative boundary evidence is insufficient — by deriving the blindness mechanically
first, admitting a challenger only on its mechanism, and running both nulls over the whole gauntlet.

`### Phase 1` was written and committed **before any challenger was implemented or run**. `### Phase 2`
was written in the same commit, after Phase 1 and before execution.

## Acceptance criteria

- Phase 1 recorded before implementation, with rejected candidates named.
- The metric, families, generation, expectations, pass rule, and counts frozen and verifiable by diff.
- Both nulls over all seven scored families on identical trials, per-family distributions reported.
- Counterexamples inspected; any post-hoc correction labelled and kept beside the original.
- A verdict distinguishing A / B / C, plus a domain-neutrality assessment.
- `scripts/check.sh` passes unweakened; no existing test changed; nothing pushed.

### Phase 1 — derivation, before building

#### 1.1 What the incumbent null preserves and destroys

`order_null_seeded` permutes the `mark` field across every event of one sequence, by fixed-seed
Fisher–Yates.

| preserved | destroyed |
|---|---|
| the **multiset of marks of the whole recording** | which mark sits at which index — all order information, globally |
| every `gap_from_previous_ms` and `offset_ms`, exactly | nothing about timing |
| the recording's length, and the span's *positions* | |
| therefore the span's entire timing profile, in every realization | |

So under this null, a span of length `L` receives an **ordered sample without replacement from the
recording's global mark multiset**, placed at its own positions, with its own gaps.

#### 1.2 Why Family E is unidentifiable to it

The statistic sprint:11 and sprint:12 scored is `z = (null_mean − observed) / null_stddev` on the
combined distance, over an ensemble in which **both** sides are permuted independently.

The observed event under that null is *"two independently permuted spans agree at least as well as the
observed pair did"*. For an exact-agreement candidate that probability is

```text
P = Σ over ordered sequences s of  P_A(s) · P_B(s)  ≈  c^L,     c = Σ_m p_A(m) · p_B(m)
```

**and that expression does not mention the observed span at all.** It is a function of `L` and the two
recordings' marginals. Two spans of the same length in the same pair of recordings have the same null
tail whatever marks they contain — so novel and redundant are the same question to it, and so are rare
and common at the level of one span.

This also corrects a claim sprint:11 made. Its Result explained the mechanism as *"a mark occurring
once in 169 events almost never lands in your window, so a span containing it is hard to match by
chance"*. That is the wrong conditioning: the double permutation never has to land that mark anywhere
in particular, because it is not being asked to reproduce anything. The residual effect sprint:12
measured in the rare-versus-common family at median `+0.073` is a **whole-recording** effect — injecting
a mark into the background at 35% raises `c` for every candidate in that recording — not a span-content
effect. That is why it was an order of magnitude smaller than the informative family's, and why its
first quartile sat below zero.

**A second, subtler point, recorded because it changes what a repair must do.** The information about
redundancy is not entirely absent from the null; it is absent from the *statistic*. Under sampling
without replacement, reproducing a span that repeats a mark of multiplicity `m` is combinatorially
*easier* than reproducing one with `m` distinct marks. But `z` reads the null's **centre and spread**,
and both are dominated by the marginal collision structure, which barely moves. `empirical_p` reads the
tail, where the information is — and sprint:11 already measured that it saturates at the ensemble floor.
So the information exists at a scale of `1e-8`, and neither statistic can reach it.

#### 1.3 The minimum conditioning a challenger needs

From 1.2 the defect is precise: **the null is asked an unconditional question.** To make span content
matter, the null must be asked to *reproduce something observed*. The minimal change is therefore to
**hold one side's observed span fixed and permute only the other**, and to ask how surprising it is
that the permuted side matched it as well as the real one did.

That single change makes the observed content enter the probability, because the event being scored now
names it.

#### 1.4 Candidate constructions considered, and the grounds for rejecting five

| candidate | verdict |
|---|---|
| **Within-span permutation** — permute marks inside the observed span only | **Rejected.** It separates novel from redundant (12 arrangements against 24) but destroys the background entirely, so a span of ubiquitous marks and a span of unique marks become indistinguishable. It repairs E by discarding the capability the rare-versus-common family exists to test. |
| **Local/windowed permutation** — permute within the span plus a surrounding window | **Rejected.** The window width is a free parameter with no principled value, and choosing one against the gauntlet is threshold tuning. |
| **Paired permutation** — apply the same permutation to both recordings | **Rejected.** It does not condition on observed content, so 1.2 applies unchanged. |
| **Inverse-frequency or `−log p(mark)` weighting of substitutions** | **Rejected on the round's own constraint,** and independently: it is a weight chosen by hand, not a probability derived from a stated null. |
| **Analytic tail of the incumbent double permutation** | **Rejected.** Exact evaluation removes the Monte-Carlo saturation but not the blindness: by 1.2 the quantity itself does not depend on span content. |
| **Conditional match surprisal** — hold one side's observed span fixed, permute the other, score the exact probability that the permuted side agrees with the observed span at least as well as the real one did | **Admitted.** See 1.5. |

#### 1.5 The admitted challenger

**Conditional match surprisal.** Same null — a uniform permutation of one recording's marks — asked a
conditional question.

Let `a[0..L)` be one side's observed span marks, `b[0..L)` the other's, and
`k = #{i : a[i] = b[i]}` the observed positional agreement. Under the null the permuted B-span is an
ordered sample without replacement of length `L` from B's whole mark multiset. Define

```text
S(A→B) = − ln P( a nulled B-span agrees with the observed a in at least k positions )
```

and symmetrize: `S = ½[S(A→B) + S(B→A)]`, in nats.

**It is computed exactly, not sampled.** For a position set `T`, the probability that a nulled span
carries `a`'s marks at exactly those positions is

```text
f(T) = ∏_m (c_m)_{t_m} / (N)_{|T|}
```

where `(x)_n` is the falling factorial, `c_m` is mark `m`'s count in the recording, `t_m` its count
among `{a[i] : i ∈ T}`, and `N` the recording's length. Then by Jordan's formula

```text
P(≥ k) = Σ_{j ≥ k} (−1)^{j−k} · C(j−1, k−1) · Σ_{|T| = j} f(T)
```

which is exact and, at `L ≤ 9`, at most 512 subsets of arithmetic. No sampling, no ensemble, no
realization count, no saturation.

**Why it is admissible.** No weight is chosen: the mark counts enter through an exact probability under
a null that already existed. No threshold, no window, no free parameter of any kind. It uses only event
counts and positional agreement, so it is defined over any sequence of timestamped categorical events
and mentions nothing about tools, agents, or this project.

**Two costs, stated before it is run.** It is defined only for **equal-length spans**, because a
positional-agreement count needs a positional correspondence; a candidate whose alignment contains an
insertion or deletion has no `k` and the statistic is undefined, which will be counted and reported. And
it **ignores timing entirely** — it is a statistic about categorical agreement, where the incumbent's is
about the combined distance. That is a genuine narrowing and it is not repaired here.

#### 1.6 The derivation, checked arithmetically before any specimen ran

On a context-40 gauntlet recording, `N = 84`, `L = 4`:

```text
P(null reproduces the novel span     Core0·Core1·Core2·Rare )  = 2.159e-08
P(null reproduces the redundant span Core0·Core1·Core2·Core0)  = 4.319e-08     ratio 2.00
                                                       ΔS = ln 2 = 0.6931 nats

P(null reproduces a rare   boundary)                          = 2.159e-08
P(null reproduces a common boundary, count 29 of 84)          = 6.262e-07     ratio 29.0
```

So the mechanism predicts the redundant extension is **less** surprising by exactly `ln 2`, and the
common boundary less surprising by `ln(count)`. Both are consequences of the falling factorial, neither
is a designed term, and both were computed before the challenger was implemented.

### Phase 2 — preregistration

#### 2.1 Frozen

Verifiable by `git diff` over `src/experiment/event_sequence.rs` (empty) and over the family
construction, expectations, and scoring in `src/experiment/gauntlet.rs`: the eight `Family` variants,
`specimen`, `competing_specimen`, `background`, `core_steps`, `ndjson`, `grid`, `REALIZATIONS` (1 000),
`assemble`'s pass rule, and every family's recorded expectation. 300 trials, unchanged.

The challenger is added as a **second statistic over the same trials**. Nothing about how a trial is
generated or scored moves.

#### 2.2 What is compared

Both statistics on identical trials, scored by the identical rule:

| family | incumbent quantity | challenger quantity |
|---|---|---|
| informative | `Δz` | `ΔS` |
| noise | `Δz` | `ΔS` |
| rare vs common | `Δz(rare) − Δz(common)` | `ΔS(rare) − ΔS(common)` |
| redundant | `Δz(novel) − Δz(redundant)` | `ΔS(novel) − ΔS(redundant)` |
| accidental | `z(planted) − z(chance)` | `S(planted) − S(chance)` |
| diluted | best-`z` candidate overlaps the core | best-`S` candidate overlaps the core |
| competing | `z(long rare) − z(short common)` | `S(long rare) − S(short common)` |

#### 2.3 Predictions

**P1 — redundant, the failure under repair.** The challenger PASSes, with a median near `+ln 2 ≈ 0.69`
nats, because §1.6 derives exactly that.

**P2 — rare versus common.** The challenger PASSes and *more strongly than the incumbent*, with a median
near `ln(count of the common mark) ≈ ln 29 ≈ 3.4` nats against the incumbent's `+0.073`.

**P3 — informative.** Both PASS. The challenger should also be positive: the added boundary mark is
unique, so it multiplies the exact probability by roughly `1/N`, raising surprisal by about `ln N ≈ 4.4`
nats.

**P4 — noise, where the challenger must not win for the wrong reason.** Adding a *mismatched* boundary
lowers the observed agreement count from `L` to `L`, so the extended span's `k` is `L` on the core and
`L` on the extension minus one — the tail widens by a large factor and surprisal should **fall**. Both
nulls should PASS. If the challenger fails here, it is rewarding length rather than evidence.

**P5 — accidental and competing.** Both PASS under both. A chance match between independent streams is
built from common marks, so its exact probability is high and its surprisal low.

**P6 — diluted, where the challenger may legitimately lose.** As context grows, `N` grows and `(N)_L`
grows, so surprisal rises for *every* candidate in the recording. Whether the argmax stays on the motif
is not obviously preserved, and a FAIL here would be a real cost of the repair rather than a defect.

**P7 — undefined candidates.** The challenger is undefined wherever an alignment contains an indel. In
the fixed-length families that should be nowhere; in the `diluted` and `accidental` families, which
search over spans of differing length, it may be common. The count is reported per family, and a family
whose trials are mostly undefined is reported as such rather than scored on the remainder.

#### 2.4 Verdict criteria

- **A — current-null limitation.** The challenger repairs redundant (PASS) **and** retains every family
  the incumbent already passed, with no family regressing from PASS to MIXED or FAIL.
- **B — deeper insufficiency.** The challenger fails to repair redundant, **or** repairs it while
  regressing one or more previously passing families, **or** requires anything Phase 1 would have
  rejected.
- **C — inconclusive.** The comparison cannot discriminate — for instance if the challenger is undefined
  on most trials of the families that matter.

If the evidence supports a more precise reading than these three, it is reported instead of being forced
into one of them.

Separately and regardless of verdict: an assessment of whether the surviving machinery is still a
domain-neutral primitive over arbitrary timestamped categorical event sequences.

#### 2.5 What this task will not do

No selector. No change to the alignment metric or its constants, the families, their generation, their
expectations, the pass rule, or the counts. No inverse-frequency weighting, hand-added rarity bonus, or
threshold tuned against results. No WitnessGlass or Claude semantics in the statistical machinery. No
weakening of Family E. No new facet, no variable-length discovery, no corpus, no fourth real specimen,
no product CLI surface, no dependency, no Spectroscope change. No real recording committed, copied, or
reproduced. Nothing pushed.
