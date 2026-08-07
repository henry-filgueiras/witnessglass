---
id: tsk_01KZEXW8YFHMGXPR5785CSJA11
sequence: 31
kind: task
status: closed
sprint: spr_01KZEXTYYH9AQX2M7JM5YCA63M
created: 2026-08-07
closed: 2026-08-07
---

# Build and run the local corpus report

## Objective

Build `corpus-report`: a local, on-demand, deterministic workflow that discovers a directory of
recordings, replays and validates each through WitnessGlass, projects them into a named observed
tool-action stream, runs the existing cross-recording search machinery over the eligible corpus,
assembles cross-session candidate families, calibrates them against the exact first-order null on the
projection they were found in, and renders a plain-language field report from a machine-readable facts
document. Then run it against a real, untracked recordings directory and read the report as a human.

**This is an exploratory engineering round.** No section below is a preregistered criterion under
decision:7, and no result of this task may be cited as an experimental finding.

## Acceptance criteria

### What must be built

- `src/experiment/corpus.rs`, `examples/corpus-report.rs`, `tests/corpus.rs`. Nothing in the product
  depends on any of them, and deleting the three deletes the capability.
- One documented command taking a recordings directory, a corpus label and an output directory.
- Discovery through `std::fs::read_dir` sorted by file name; replay and validation through
  `replay_file` and `inspection::inspect`. No second reader of raw NDJSON.
- `facts.json`, `manifest.json`, `report.md`. `report.md` is rendered from the facts document, and a
  `--render-from` mode proves it by regenerating the report from facts alone.
- `manifest.json` deterministic: no wall clock. Volatile run metadata, if any, lives in a separate
  file.
- A `--compare` mode consuming two facts documents and writing `comparison.md`.

### What must be true of the analysis

- Every discovered file appears in the manifest as included or skipped, with a reason, under an opaque
  identity.
- The workflow projection derives one action per correlated tool group, in canonical order of each
  group's earliest record, carrying: delivered tool names, the group's shape, its observed terminal
  outcome class, its derived category, and its first and last raw sequence numbers.
- `GroupShape::ReportedIntentOnly` groups contribute no action, and are counted and reported as
  excluded rather than dropped.
- Candidates come from `cross_pairs` + `dedupe_overlapping` + `Observation` + R1 — the established
  machinery, called and not modified — and are retained with both sessions, both window starts, the
  span length, the alignment decomposition, the R1 score, the mark sequences, and receipt ranges.
- A family is an **exact** mark-label sequence. Only a candidate whose two windows carry identical
  mark sequences establishes cross-session support; approximate candidates are retained and counted
  but never contribute prevalence. Exact grouping admits no transitive chaining.
- Prevalence is `N of M eligible sessions`, with M stated. Non-overlapping occurrence counts are
  reported beside it.
- The calibration runs the same discovery statistic on `doublet_null_seeded` replicates of **every**
  eligible session, at the same span length, and compares the observed family's session count against
  the distribution of the null corpus's **best** family. Family-wise by construction.
- The raw event projection runs as a control under the identical procedure, and shapes it explains are
  quarantined.

### What must be tested

- A synthetic corpus in which a planted shape recurs across several sessions is recovered, named, and
  ranked.
- A negative corpus of pure request→outcome alternation, and one dominated by a single repeated tool,
  produce no unquarantined lead.
- Tiny, empty, corrupt and truncated inputs are skipped with the right reason and never disappear.
- Ranking and serialization are deterministic; two runs over the same input produce identical bytes.
- The retained search agrees with `calibration::complete_search` on the maximum R1 it keeps, so the
  new retention cannot drift from the established statistic.
- A synthetic A/B comparison in which one family is gained and one strengthened.

### Gates

`scripts/check.sh` green and unweakened. No recording committed. No real-corpus output committed. No
prompt, response, command, output, path, or host identity in any artifact. Nothing pushed.

## Result

**Built and run.** `cargo run --release --example corpus-report` produces `report.md`, `facts.json`,
`manifest.json` and `run.json` from an arbitrary directory of recordings; a second run over the same
directory produces the first three **byte for byte**, verified mechanically on both a synthetic corpus
and the real one. decision:9 records the authorization and the boundaries. 434 tests, up from 417; 17
new in `tests/corpus.rs`.

**This round is exploratory engineering, not an experiment.** Nothing above was a criterion under
decision:7, no verdict partition was declared, and no number here may be cited as an experimental
result.

### 1. What was built

`src/experiment/corpus.rs`, `examples/corpus-report.rs`, `tests/corpus.rs`. `src/main.rs` still does
not reference `crate::experiment`; deleting the three files and the one `pub mod corpus;` line removes
the capability. No product surface changed, and **no line of `event_sequence`, `calibration`,
`identifiability`, `repair` or `transition_null` was touched** — the search is called, not modified.

### 2. The workflow projection

One [`Action`] per correlated tool group, in canonical order of each group's earliest record. The mark
is the group's **terminal-outcome event kind** — a kind read off a record that exists — plus a derived
[`Category`]. So a failed `Verify` and a successful one are different marks, and a failure-recovery
shape survives into the search rather than being flattened.

`GroupShape::ReportedIntentOnly` contributes no action and is counted separately. No reported text is
read anywhere in the module. Incomplete, denied, failed, ambiguous and outcome-without-opening cases
are preserved as distinct outcomes with their receipts.

**The category vocabulary is version 1 and small on purpose:** `Inspect`, `Modify`, `Verify`,
`VersionControl`, `Research`, `Delegate`, `Shell`, `Other`. A shell call is classified by its **leading
program name only**, with one concession — a command opening with `cd` is read from the token after its
first `&&`. Only the category and the sequence number survive; the command string is dropped in the
same function that reads it, and `tests/corpus.rs` asserts no marker planted in a synthetic command
reaches `facts.json`, `manifest.json` or `report.md`.

`Shell` is 35% of the real corpus. That is the honest cost of refusing to guess, and the report says so
rather than inventing labels to shrink it.

### 3. Candidates and families

Discovery is `cross_pairs` + `dedupe_overlapping` + `Observation` + R1 — the established pipeline, with
`keep = 40` per pair per span length over `{3, 4, 5, 6}`. `retained_search` keeps what
`SearchOutcome` discards: both sessions, both window starts, span length, alignment decomposition, R1,
both mark sequences, and receipt ranges. **A test asserts `retained_search` and
`calibration::complete_search` read the same maximum R1 at `KEEP`**, so the retention cannot drift from
the established statistic.

A family is an **exact** mark sequence. Only a candidate whose two windows carry identical marks
establishes cross-session support, so there is no near-miss to chain through and `A≈B`, `B≈C` cannot
imply `A≈C`. Prevalence is recounted by exact non-overlapping scan over every eligible session, and is
reported as `N of M` with `M` stated in the report itself.

Two reporting folds, both **flags rather than deletions** so two corpora's facts documents stay
comparable: a shape that is a contiguous fragment of a longer shape holding exactly the same sessions
is `subsumed_by` it; a shape over the same *set* of steps as a better-supported one is `variant_of` it.
The second fold was added after reading the first real report, where four permutations of the same
three steps occupied four of five lead slots.

### 4. The calibration, and what it is not

Per span length, every eligible session is replaced by a `doublet_null_seeded` replicate of itself —
sprint:20's exact first-order null, unchanged — and the same discovery statistic is recomputed on the
null corpus inside every replicate. The statistic is the **best shape's session coverage**, so the test
is family-wise by construction: the null takes its own maximum over every shape it could have found.

**This is not sprint:19's `T`,** and the report says so in as many words. sprint:20's collapse was
measured on the raw event projection; it is not transferred here by assertion, and the workflow
projection carries its own null or its output is labelled descriptive.

### 5. The real corpus

`~/cuecraft/.witnessglass/recordings`, run at `B = 999` in 17 seconds. **12 files discovered, 10
analysed, 2 set aside** — both for holding fewer than 12 observed actions (two records each). No
recording was truncated. 1750 actions: 1726 succeeded, 22 failed, 2 denied, 0 unpaired. 962
`reported_intent` records, none read.

**No shape cleared the calibration, at any span length, on either projection.** The lowest tail on the
workflow projection is 0.692. Two of the four span lengths are **saturated** — the reshuffled corpora
already put some shape in all ten sessions more than half the time — and the report says that outright
rather than reporting a tail of 1.000 as if it meant something.

The descriptive lane is where the value is. Top leads by session coverage:
`Shell → Inspect → Modify` (7 of 10), `Modify → Verify → Shell` (6), `Shell → Inspect → Shell → Verify`
(5), `Inspect → Modify → Verify` (4), `Modify → Verify → Shell → Inspect` (4). The first stands for 60
other shapes over the same three kinds of step — which is itself the finding: **at these lengths, which
steps co-occur carries far more than the order they occur in.**

The raw projection did its job as a control: 243 exact cross-session shapes, 225 quarantined, and its
best-supported shapes are pure `tool_requested Bash → tool_succeeded Bash` alternation in all ten
sessions. That is the recorder, not the agent.

**One degeneracy worth recording.** `6a8a02cc`'s exact null returns the observed sequence unchanged in
**434 of 999** replicates — the same failure mode sprint:20 recorded as D5, at four times the rate. Its
contribution to every null distribution here is partly a comparison against itself. Reported in the
report's own limitations, not buried.

### 6. decision:8, resolved deliberately

**No specimen was admitted.** decision:8's inventory is unchanged. The cuecraft directory now holds
twelve recordings where that decision's inventory reflects three — sprint:20 §D6 recorded the same
divergence at six — and this round did not close it either, because admitting a specimen is an
amendment to decision:8 and not a line in a Result. decision:9 records that a corpus report over an
untracked directory is a **local exploratory artefact**, and no finding from it has entered public
archaeology: the numbers above are counts, shares and prevalences, which decision:8 already permits.

Generated output lives at `.witnessglass/corpus-reports/cuecraft/`, which `.gitignore` already covers.
Nothing from it is committed.

### 7. Verified

Synthetic recovery: a four-step shape planted in every session of a five-session corpus is found, named
`Inspect–Modify–Verify–Inspect loop`, reaches 5 of 5, keeps its receipts, keeps its underlying
`Read → Edit → Bash → Read` tool sequence, and reaches a human as a lead. Negative controls: a corpus
over a two-symbol vocabulary produces **no** unquarantined lead and the report says so in plain words;
raw-projection request→outcome alternation is recognised as protocol. Eligibility: empty, corrupt,
tiny, single-category, truncated and duplicate-identity inputs each reach the manifest with the right
reason. Determinism: two analyses byte-identical across facts, manifest and report; `--render-from`
reproduces `report.md` from `facts.json` alone. A/B: a synthetic pair with one gained and one
strengthened shape, both denominators printed.

`scripts/check.sh` green and unweakened. Nothing pushed.

### 8. What this round does not establish

That any shape here is a motif, a habit, or a property of coding agents. That the categories mean what
their names suggest. That a second corpus would look like this one. That the absence of a calibrated
result is evidence of an absence of structure — two of four span lengths could not have produced one,
and the report says which.

**The honest summary is that the eyepiece works and the corpus is the binding constraint**, which is
the same conclusion sprint:20 §11 reached from the other direction.

### 9. Scarp desire paths

**idea:1 recurred.** This Result was appended to the task file with a shell write before
`scarp close task:31`, because closing and recording a result remain two writes and only one is a Scarp
command.

**One new piece of friction, and it is small.** `scarp new task --body-file` **refused** a body
containing a `## Result` heading — correctly, since Scarp owns the template and a task has no `Result`
section until it is closed. The refusal was clear and named the available sections, which is exactly
what a good error does. The friction is only that the section a task will certainly grow is
unwritable at creation time and unmentioned until you try, so the natural move — drafting the whole
artifact in one file — fails once per round for anyone who has not learned it. `scarp sections task`,
or the same list in `scarp new --help`, would remove it. Not worth an idea on its own; noted here in
case it recurs.
