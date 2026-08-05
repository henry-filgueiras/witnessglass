---
id: tsk_01KZA0ZWNTWT60R828A01RSSJH
sequence: 23
kind: task
status: closed
sprint: spr_01KZA0ZWNEG3KMC464BN7CWNKD
created: 2026-08-05
closed: 2026-08-05
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

## Result

Delivered. **B — deeper insufficiency**, with a reading more precise than the label, and with one real
gain that the label hides.

Three sentences:

> The minimal conditional challenger **repairs the rarity mechanism decisively** — the family that
> limped at 0.700 and `+0.073` now runs 30 of 30 at `+3.03` nats — and **does not repair Family E**.
> It does not fail to *see* the redundancy: `S(core)` is lower by exactly `ln 2` in the redundant arm.
> It fails because that same `ln 2` discounts the core and the extension equally and **cancels in the
> delta**, so Family E's question is not answerable by any statistic that is a function of a span and a
> recording's marginals and is scored as a difference between nested spans.

### 1. Frozen, verified

`git diff 61cf0d7..HEAD -- src/experiment/event_sequence.rs` is **empty**. The eight families, their
construction, their expectations, the pass rule, the grid, and `REALIZATIONS` are unchanged in
`gauntlet.rs`; the diff there adds a second statistic over the same trials and nothing else.

The strongest guarantee is a test rather than a diff: `adding_the_challenger_left_the_incumbents_
numbers_exactly_where_they_were` pins sprint:12's seven incumbent rows — trials, fraction, median, and
verdict — to within `5e-4`. If a later round perturbs the incumbent while claiming only to add a column
beside it, that fails.

### 2. The comparison

300 trials, identical for both columns, scored by the identical rule.

| family | statistic | trials | undef | frac | median | verdict |
|---|---|---|---|---|---|---|
| informative | z | 60 | 0 | 1.000 | +0.515 | PASS |
| informative | **surprisal** | 60 | 0 | **1.000** | **+4.054** | **PASS** |
| noise | z | 60 | 0 | 1.000 | −0.770 | PASS |
| noise | **surprisal** | 60 | 0 | **1.000** | **−3.487** | **PASS** |
| rare vs common | z | 30 | 0 | 0.700 | +0.073 | PASS |
| rare vs common | **surprisal** | 30 | 0 | **1.000** | **+3.034** | **PASS** |
| redundant | z | 30 | 0 | 0.467 | −0.003 | **FAIL** |
| redundant | **surprisal** | 30 | 0 | **0.333** | **−0.000** | **FAIL** |
| accidental | z | 30 | 0 | 0.967 | +2.403 | PASS |
| accidental | **surprisal** | **12** | **18** | 1.000 | +10.926 | PASS |
| diluted | z | 40 | 0 | 1.000 | +1.000 | PASS |
| diluted | **surprisal** | 40 | 0 | **1.000** | +1.000 | PASS |
| competing | z | 20 | 0 | 1.000 | +0.427 | PASS |
| competing | **surprisal** | 20 | 0 | **1.000** | **+17.178** | **PASS** |

`z` is dimensionless and surprisal is in nats, so the two medians are not comparable in magnitude. The
fractions are, and they are the row to read.

**No family regressed in verdict.** One regressed in *coverage*: the accidental family drops from 30
scored trials to 12, because it searches over spans that may differ in length and the challenger is
undefined across an indel. Predicted as P7 and it happened exactly there and nowhere else.

### 3. The gain, which is real and was not the round's target

**Family C/D is the round's incidental repair.** The two arms are identical by construction in marks,
gaps, core, and seed — a sprint:12 test asserts their raw agreement matches to `1e-6` — and differ only
in whether the boundary mark also appears in the background at 35% prevalence.

```text
incumbent   0.700 of pairs, median +0.073, first quartile −0.031
challenger  1.000 of pairs, median +3.034, first quartile +2.669
```

Predicted in §1.6 as `ln(count of the common mark)`: `ln 15 ≈ 2.7` at context 20 and `ln 29 ≈ 3.37` at
context 40, so a median near 3.0 across the mixed grid. Measured **3.034**.

This matters beyond the family. task:23 §1.2 argued that sprint:11's stated mechanism was the wrong
conditioning and that its rare-versus-common effect was a whole-recording artefact rather than a
span-content effect. **That argument is now measured**: put the same null under a conditional question
and the span-content effect appears at forty times the magnitude, unanimously.

### 4. The failure, diagnosed to machine precision

Prediction P1 said the challenger would repair Family E with a median near `+ln 2 = 0.693`. It came out
at `−0.0000`, 10 of 30 in the expected direction.

The reason, measured on matched pairs:

```text
                 S(core)      S(expanded)     ΔS
novel arm        11.2831      14.9966         3.7136
redundant arm    10.5899      14.3035         3.7136
                 └── differ by exactly ln 2 = 0.6931 ──┘        ΔS difference:  0.0000
```

**The challenger does see the redundancy.** `S(core)` — of a span that is literally the same three marks
in both arms — is lower by exactly `ln 2` in the recording where `Core0` occurs twice. §1.6's arithmetic
was right.

**And it sees it in the wrong place.** The extension costs the same in both arms, because what an added
event costs is a factor of *(unused copies of its mark remaining) / (N − L)*, and in both arms exactly
one unused copy remains: the novel arm's `Rare` is the first of one, the redundant arm's second `Core0`
is the second of two. So the `ln 2` discounts the core and the extension identically and vanishes from
the difference.

**The general statement.** Family E's question — *does a boundary that repeats information already in
the core buy less evidence?* — is about **information the span already carries**. What a permutation
null can express is **how many copies the recording still has available**. Those coincide only when the
recording holds more copies than the span uses, and in Family E it holds exactly as many as the span
uses, in both arms. No statistic that is a function of *(the span, the recording's marginals)* and is
scored as a difference between nested spans can separate them.

**This is a diagnosis and not grounds to change Family E**, which stands exactly as sprint:12 wrote it.

### 5. My Phase 1 derivation was arithmetically right and applied to the wrong quantity

§1.6 computed `P(reproduce the redundant span) = 2 × P(reproduce the novel span)` and concluded that the
challenger would show `ΔS ≈ ln 2`. The first half is correct and is now pinned by a test. The second
half does not follow: the family scores a *delta between nested spans*, and the factor of two appears in
both terms.

I derived the mechanism, wrote the number down before running, and drew the wrong conclusion from my own
arithmetic — because I compared the two expanded spans and never asked what the same factor did to the
two cores. **Seventh defect in eight rounds, and a new shape:** not a missing check, an unreachable
cutoff, an inherited threshold, an unapplied mechanism, an under-specified statistic, or a ladder that
does not tile — this time a correct calculation about the wrong quantity.

The generalizable step, recorded and not built: **when a criterion scores a difference, do the algebra
on the difference, not on either term.**

### 6. Counterexamples and near-failures

**Redundant, all thirty.** Every value is `±0.0000` to four places. There is no distribution to inspect;
the two arms are identical by an exact cancellation. That is a cleaner failure than sprint:12's noisy
one and it is what made §4's diagnosis findable.

**Accidental's eighteen undefined trials** are the only coverage loss, and they are all the same
condition: the frozen boundary search returned a best candidate whose two spans differ in length, so
there is no positional correspondence and no agreement count. Asserted by test to be the *only* reason
the challenger declines.

**No conditioning leakage found.** The challenger never sees a family label, a planted span, or a trial
parameter; it receives two sequences and two spans. Its inputs are mark counts and positional agreement,
and a test checks the closed form against brute-force enumeration of every permutation on five small
populations at every value of `k`.

**A defect the brute-force test found**, before any specimen ran: `agreement_tail` underflowed at
`k = 0`, where Jordan's formula's `C(j−1, k−1)` is undefined. `P(≥ 0) = 1` is now returned directly. No
production path reaches `k = 0`; only the exhaustive test did.

### 7. Verdict: **B — deeper insufficiency**, precisely stated

By task:23 §2.4: *"the challenger fails to repair redundant"* → **B**. Not A, which requires the repair.
Not C: the experiment discriminated sharply and to machine precision.

The more precise reading the criteria invited:

> Family E is not a question about the null. It is a question about information already carried by a
> span, and permutation nulls express availability in the population instead. Conditioning the null
> repaired a different defect — the span-content sensitivity sprint:11 claimed and sprint:12 measured as
> weak — decisively and at forty times the effect size. The redundancy blindness survived because it was
> never the null's to fix.

**Where the epicycles would start, and why the round stops here.** A statistic *could* separate E's arms
by excluding the core's own events from the population before scoring the extension — a second
conditioning layer, added because the first did not work. That is the shape of an epicycle: a correction
motivated by a residual rather than by a mechanism, and the round declines to take it. If it is ever
taken, it should be preregistered as such and judged on whether it survives the whole gauntlet, not on
whether it rescues one family.

### 8. Is the surviving machinery still domain-neutral?

**More so than the incumbent, and narrower.**

The challenger consumes: a sequence of categorical labels, their counts, and a positional agreement
count between two equal-length windows. It contains no tool name, no channel, no adapter, no schema
version, no notion of an agent, and no WitnessGlass or Claude vocabulary of any kind. It is a statistic
over **arbitrary categorical sequences** — and notably it does not use the timestamps at all, so it does
not even need the "timestamped" qualifier the incumbent does.

That is also the narrowing. The incumbent's `z` is computed on the combined distance, which carries the
timing policy; the challenger is blind to timing entirely. A boundary that agrees in identity and
disagrees wildly in spacing is invisible to it. Stated in §1.5 before it was run, and unrepaired.

So: the machinery is a domain-neutral primitive, and the price of that neutrality is that it answers a
strictly smaller question than the thing it was built to challenge.

### 9. Desire-path friction

**Eighth consecutive round with the preregistration in a `###` subsection**, and this one carried a
derivation the whole round depended on. `61cf0d7` contains nothing else. **idea:5**.

**A new observation about that gap.** This round's Phase 1 was an *analysis* commit — reasoning, not
prediction — and Scarp has no place for it either. It went into the same `###` block as the
preregistration, which means the record cannot distinguish "this is what we worked out" from "this is
what we committed to before looking". They are different epistemic objects and this round needed both.
Recorded beside idea:5 rather than as a new idea, since the affordance that would serve it — a section
stamped when written — is the same one.

**Appending a Result is still `cat >>`** — `scarp` 0.2.0, version lag, maintenance:1.

**One thing that went well.** The brute-force test against exhaustive permutation enumeration found a
real bug in the closed form within minutes of being written, on a code path no specimen reaches. Exact
statistics are testable in a way sampled ones are not, and that is an argument for the challenger's
construction independent of its results.

### 10. Strongest limitation

**One challenger was admitted and six were rejected, all on reasoning rather than measurement.** §1.4's
rejections are argued and I believe them, but they are unmeasured: within-span permutation was rejected
for destroying the background, and *how much* it would have destroyed was never quantified. A round that
admits exactly one candidate cannot report how close the runners-up were.

Secondly, the challenger's coverage gap is real and load-bearing for any future use: it is undefined
wherever an alignment contains an indel, which is 60% of the one family that searches over variable
spans. A statistic that cannot score variable-length candidates is a poor foundation for variable-length
boundary discovery, which is where this line of work was heading.

### 11. Recommendation: exactly one next experiment

**Stop adding statistics and test whether the representation can carry the question at all.**

Family E asks about information already in the span. §4 shows no marginal-based permutation statistic
scored as a nested delta can express that, and §7 shows the repair that would is an epicycle. The
question that remains is whether the *representation* — a mark being a schema-tagged kind plus a verbatim
tool name — is what makes redundancy invisible, or whether redundancy is genuinely not a property these
recordings carry.

The smallest test: take sprint:12's Family E specimens unchanged, and measure whether **any** function
of the two spans' marks alone can separate the arms — by exhaustive enumeration over a small family of
candidate functions rather than by inventing one. If none can, redundancy is not in the representation
and the next lever is a richer mark. If some can, the question is which, and why a permutation null is
not among them.

**Not recommended:** a third null, a second conditioning layer, or a selection policy. The verdict
forbids the last, §7 names the second as an epicycle, and the first has now been tried.

### What this task did not do

No selector. No change to the alignment metric or its constants, the families, their generation, their
expectations, the pass rule, or the counts — the diff over `event_sequence.rs` is empty and a test pins
the incumbent's seven rows. No inverse-frequency weighting, hand-added rarity bonus, or threshold tuned
against results. No WitnessGlass or Claude semantics in the statistical machinery. No weakening of
Family E, which stands exactly as sprint:12 wrote it despite this round explaining why it cannot be
satisfied. No new facet, no variable-length discovery, no corpus, no fourth real specimen, no product CLI
surface, no dependency, no Spectroscope change. No real recording committed, copied, or reproduced.
Nothing pushed.
