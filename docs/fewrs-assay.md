# Reproducing the sprint:22 FewRS retrospective assay

A runbook for task:32. Everything here is deterministic: the null, the seed schedule, the search and
the statistic are sprint:19's, unchanged, and the only new input is the replicate budget.

**Nothing in this document is a product surface.** `event-motif` is a disposable research example;
`witnessglass --help` does not mention it and `src/main.rs` does not reach it.

## What it computes

For each `(specimen, k)` cell it runs the unchanged complete search — every window pair at `k`,
alignment ranking, greedy deduplication to five, R1 readout, maximum — on the observed pair, then
reruns that entire search inside each of `m` order-null replicates, and certifies the cell iff

```text
observed T_k  >  max over the m null T_k
```

Ties do not certify. An undefined `T_k` does not certify. An empty null set does not certify.

`m` is derived, not configured: `m = ceil(ln(1/alpha) / ln(1/(1-alpha)))`, which at
`alpha = 0.01` is `459`. `--replicates` overrides it for diagnostics only; the preregistered run
leaves it alone.

## The controls

Run first, always, and by construction: `--fewrs` executes the negative control and the positive
control before it replays a single recording, and refuses to run the observational cells if either
fails its rule. Neither control needs a recording, so this reproduces without any corpus:

```bash
cargo run --release --example event-motif -- --fewrs
```

## The full assay

The four specimens are decision:8's admitted inventory. Three live in this repository's untracked
recordings directory and one in cuecraft's. **Supply them in this order** — the pair labels in the
output, and therefore the join against sprint:19's frozen grid, are built from the session prefixes in
the order given:

```bash
cargo run --release --example event-motif -- --fewrs \
  --corpus .witnessglass/recordings/8b68dece-*.ndjson \
  --corpus .witnessglass/recordings/57f18ff9-*.ndjson \
  --corpus .witnessglass/recordings/f5c18299-*.ndjson \
  --corpus <CUECRAFT>/.witnessglass/recordings/7d95c414-*.ndjson \
  --json
```

### The output contract

**With `--json`, stdout carries exactly one JSON document and nothing else.** The human report goes to
stderr, where it stays visible while the run is in flight without contaminating a redirected file.
Without `--json`, the human report goes to stdout as before and no JSON is produced. maintenance:3
repaired this: the first implementation printed the report to stdout and appended the document after
it, so the command below produced a file no parser accepts.

The document carries every cell's observed `T_k`, null maximum, certification, refuting-null count,
searches performed, the frozen 999-replicate tail and exceedance count, both agreement flags, the
seed-range identity, the null construction, the cost block, the eligibility envelope, and the
classification.

**Where the output goes.** Not into this repository. It is derived from real recordings, and the
repository commits no real-recording-derived documents — only the mechanically derived counts and
scores decision:8 permits, and those live in the task Result. Redirect it under `.witnessglass/`,
which `.gitignore` already covers, and keep the two streams apart:

```bash
… --json > .witnessglass/fewrs/fewrs.json 2> .witnessglass/fewrs/fewrs.log
```

`fewrs.json` is then a plain JSON file: `python3 -m json.tool < .witnessglass/fewrs/fewrs.json`
accepts it, and `tests/fewrs.rs` runs the example and parses its whole stdout on every gate.

### What gets classified, and what does not

The preregistered `STRONG` / `WEAK / MIXED` / `FALSIFICATION` outcomes belong to **one** protocol. A run
that departs from it in any of these ways is reported as `DIAGNOSTIC / UNCLASSIFIED`, with the failing
conditions named in both renderings, and is never compared against the frozen 15-of-30 threshold:

- `alpha` other than `0.01`, or a budget other than the `459` derived from it — `--replicates` always
  lands here;
- a seed range other than `null_seed(0..459, {0,1})`, or a null other than the order permutation;
- the two synthetic controls not executed over the whole ladder;
- a specimen set other than decision:8's exact four;
- an observational grid that is not exactly the frozen thirty `(pair, k)` cells — a missing, extra or
  duplicated cell, or thirty cells with the wrong identities;
- any cell that cannot be joined to sprint:19's published grid.

One exception, and it is deliberate: a run that *did* establish the protocol and whose control failed is
`FALSIFICATION`, not a diagnostic. The frozen protocol stops before the observational stage when a
control fails, so its empty grid is obedience rather than a defect.

Output is **not** redacted and is exactly as sensitive as the recordings behind it, minus what the
projection never reads. It carries opaque eight-character session prefixes, counts and scores, and no
prompt, command, response, path or payload excerpt — but rendering is not redacting.

## Cost

At the preregistered budget the assay performs `459 * 40 = 18360` complete null searches — ten control
cells and thirty observational ones — against `999 * 40 = 39960` for the same coverage at sprint:19's
replicate count. One pass took **72.9 s** in a release build on one machine; that number is
machine-specific, secondary, and decides nothing.

## Reproducing the diagnostic runs

The descriptive `m = 99` comparison in task:32's Result, which no verdict branch reads:

```bash
cargo run --release --example event-motif -- --fewrs --replicates 99 --corpus …
```

It reports `DIAGNOSTIC / UNCLASSIFIED`, naming the budget and seed-range conditions it broke. That is
correct: it is a cheaper *different* test, not a cheaper run of the frozen one, and task:32 §10 reads it
as a description rather than as a verdict. It is also the measurement that retired FewRS here — an
ordinary strict-maximum randomization test with 99 null draws has a null rejection probability of at
most `1/(99+1) = 0.01` for one exchangeable scalar statistic, and it certified 22 of 30 cells against
FewRS's 17.

## Gates

```bash
scripts/check.sh
```
