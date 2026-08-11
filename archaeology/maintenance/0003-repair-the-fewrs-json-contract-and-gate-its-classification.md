---
id: mnt_01KZSFCW71DSQCKMDXEJ86GJN6
sequence: 3
kind: maintenance
status: closed
created: 2026-08-11
closed: 2026-08-11
---

# Repair the FewRS JSON contract and gate its classification

## Work

Three defects in sprint:22's deliverables, none of which touches the frozen
result and all of which would mislead the next reader of it. Plus the
whitespace item that came in with them.

**Hand-authored, and that is worth stating.** `scarp 0.2.0` has no
`maintenance` collection — `scarp new maintenance` refuses, and `scarp
doctor` does not count these files. maintenance:1 and maintenance:2 set the
convention and this follows it. `CLAUDE.md` §7's rule that Scarp owns
sequence numbers and front matter applies to Scarp's own collections;
this one is outside them, and inventing an id by hand is the only way to
add to it.

### D1 — `--json` did not emit JSON

`fewrs_mode` printed the whole human report to stdout with `println!` and
then appended the document after it. The runbook's own documented command,

```bash
… --fewrs --json > fewrs.json
```

therefore produced a file no parser accepts: a banner, three tables, a
classification block, a cost block, and only then an object. Every
consumer of the deliverable would have had to hand-split the file, which
is exactly what happened when the round's own results were first read
back.

**Nothing committed is malformed.** `.witnessglass/` is gitignored and the
repository commits no real-recording-derived document, so the only
malformed artifact was the local run output, repaired by splitting it at
the first `{`. No observational campaign was rerun to fix formatting.

### D2 — the preregistered classification was ungated

`classify(controls_passed, certified)` read two values and nothing else,
so **any** invocation was scored against task:32's frozen 15-of-30
threshold. `--fewrs --replicates 99` — a diagnostic that is deliberately a
*different* test — printed `STRONG`. So would a run with no corpus, the
wrong specimens, a partial grid, or thirty cells carrying the wrong pair
identities.

The frozen outcomes are statements about one protocol. Emitting them for
an arbitrary run is not a rendering bug; it is the executable claiming an
experiment it did not run.

### D3 — `m = 459` was explained as the price of pooling

task:32 §PHASE 0 D11, §10 and §12, and `src/experiment/fewrs.rs`'s module
header all explained the budget as the cost of FewRS's family-wise pooled
maximum, and concluded that the budget was "over-bought" *because* this
grid cannot form a pooled maximum. The conclusion — retire FewRS — was
right. The reasoning was wrong, and a wrong reason for a right answer is
the kind of thing a later round inherits and acts on.

### D4 — trailing whitespace in `docs/fewrs-assay.md`

One line, in the diagnostic command block.

## Result

Fixed in the closeout commit. The frozen experimental result is unchanged
and was not recomputed: controls passed, 17 of 30 cells certified at
`m = 459`, classification `STRONG`, primary agreement 24/30. task:32's
Result is not rewritten; §16 was appended to it, pointing here.

### D1 — the corrected output contract

With `--json`, stdout carries **exactly one JSON document and nothing
else**; the human report goes to stderr, where it stays visible during a
73-second run without contaminating a redirected file. Without `--json`,
the report goes to stdout as before.

Implemented by choosing the report's stream once, up front, and writing
every line through a `say!` macro — so redirecting the whole report is one
binding rather than forty `println!`s, and a stray `println!` added later
is a visible anomaly rather than an invisible one.

Verified at the process level, not the struct level: `tests/fewrs.rs`
executes the built example, captures stdout, and parses the **entire**
stream with `serde_json::from_str`, which refuses trailing content. A
sibling test asserts the report is still on stdout without `--json` and is
not JSON, and four report-only markers are asserted absent from the JSON
stdout.

### D2 — one gate, comparing identities rather than counts

`fewrs::envelope(&RunDescriptor)` returns two lists of typed
`Ineligibility` reasons — one for the protocol, one for the grid — and
`classify` reads them. Rendering asks the gate; it never re-derives one.

The protocol conditions: `alpha == 0.01`; budget `== 459` and equal to what
`fewrs_budget(alpha)` derives; every cell's seed range equal to
`null_seed(0..459, {0,1})`; every cell's null construction the order
permutation; both synthetic controls executed over the whole ladder. The
grid conditions: the specimen identity **set** is decision:8's exact four;
no duplicated cell; the `(pair, k)` **set** is exactly the frozen thirty;
thirty unique cells; every cell joined to sprint:19's published grid.

Sets and identities, never `cells.len() == 30` — a run with thirty cells
carrying invented pair labels fails, and a test proves it does.

Precedence, and the middle branch is the one that needed care:

1. protocol not established → `DIAGNOSTIC / UNCLASSIFIED`;
2. protocol established, a control failed → `FALSIFICATION`, the
   preregistered outcome. Its grid is empty **because the frozen protocol
   stops there**, so emptiness must not be held against it;
3. protocol established, controls passed, grid not exact →
   `DIAGNOSTIC / UNCLASSIFIED`;
4. otherwise the frozen threshold.

A diagnostic run is never printed beside "STRONG needs >= 15", and both
renderings list the failing conditions — the JSON as tagged objects plus
rendered text, the human report as the same text, from the same `Display`
impl so they cannot drift.

`null_mode` is stamped on each cell by `cell_from` from the path it
actually took, rather than declared by the runner, so the check reads what
ran instead of what a caller claimed.

### D3 — the corrected statistical reading

`m = ceil(ln(1/alpha)/ln(1/(1-alpha)))` is the cost of FewRS's **particular
high-probability upper-bound construction**. The formula reads `alpha` and
nothing else, and **applies to a single analysis exactly as it does to a
family**. The budget is therefore not caused by pooling and does not shrink
if you stop pooling. The earlier "price of multiplicity" wording is
withdrawn.

What sprint:22 actually implemented is not FewRS's procedure but an
ordinary strict-maximum randomization test per cell, whose guarantee comes
from exchangeability alone: under the null the observation and its `m` null
statistics are exchangeable, so the probability the observation is the
strict maximum is at most `1/(m+1)`.

**Which is why 459 is the wrong number for this question.** For one
exchangeable scalar statistic at `alpha = 0.01`, `m = 99` already gives a
null rejection probability of at most `1/(99+1) = 0.01`. task:32 measured
both: the 99-draw test certified **22 of 30** cells and agreed with
sprint:19's frozen grid on **27 of 30**, against **17** and **24** for the
459-draw FewRS assay. For the narrow binary per-cell question, FewRS is
operationally dominated — 4.6x the computation, fewer certifications.

**The 99-draw alternative is not over-claimed either.** It is a per-cell
test. It confers **no** family-wise control over this heterogeneous 30-cell
grid. A pooled max-statistic test would need a coherent null dataset, a
family statistic on a commensurable or defensibly normalized scale, and its
own error-control contract — none of which this round built or defended.

**And the paper's stronger threshold guarantee is not relied on here.** Its
proof carries assumptions, subset pivotality and i.i.d. resamples among
them, that sprint:22 did not check against this pipeline. Nothing
operational should lean on it without independent statistical review. This
is a boundary, not an invitation to review the paper.

### Why STRONG and "retire" are compatible

They answer different questions, and the round was built so they could
disagree.

`STRONG` is the answer to task:32's frozen success criterion: *did at least
15 of 30 cells certify under the preregistered protocol?* Seventeen did.
That criterion was fixed before execution and is not reinterpreted here.

"Retire" is the answer to the operational question: *is this the procedure
to run?* No — a simpler 99-draw scalar randomization test delivers **more**
certifications (22 against 17) and closer agreement with the frozen grid
(27/30 against 24/30) for **4.6x less computation**, at the same nominal
`alpha`. FewRS passed its own test and still lost to a cheaper alternative.

Nothing about the second answer weakens the first. A preregistered
criterion measures what it was written to measure; an adoption decision
weighs cost, guarantee and alternatives, and the alternative here was
never in the preregistration because sprint:22 was commissioned to
evaluate FewRS rather than to survey randomization tests.

### The decision not to nominate sprint:21

task:32 §12 named sprint:21's corpus-report calibration as "the one place
the idea might still belong", on the grounds that it already forms a pooled
maximum over a family on one scale. **That nomination is withdrawn**, for
two reasons.

It rested on D3's mistaken reasoning: if 459 is not the price of pooling,
then "find something that pools" is not a reason to keep FewRS. And it
pointed at a structure this round measured nothing about — sprint:21's
calibration was read from task:31's acceptance criteria, not exercised.

What survives is narrower and is a different investigation: sprint:21's
calibration may be a candidate for an **ordinary pooled max-statistic
randomization test**, which would need its own null dataset, family
statistic and error-control contract. That is not FewRS and this experiment
established nothing about it. No sprint is opened for it here.

### One thing the process-level test had to learn

Locating the example binary is harder than it looks, and the first two
attempts both passed locally and failed under the gate. `CARGO_BIN_EXE_*`
covers `[[bin]]` targets and not examples, so the directory has to come
from `current_exe()`. Inside it sit **two wrong builds**: under
`--all-targets`, which `scripts/check.sh` uses, cargo compiles every
example a second time as a libtest harness — usually the newest file, and
it answers `--fewrs` with `Unrecognized option` and exit 101 — and builds
from earlier sessions persist, one of which predates sprint:22, prints the
example's banner happily, and then rejects `--fewrs` as an unexpected
argument.

Newest-by-mtime picked the harness. Newest-plus-banner picked the stale
build. The selector now probes `--help` for the **capability under test**
— the banner *and* the `--fewrs` flag — newest first. Worth recording
because the same trap waits for any future process-level test over an
example in this repository.

### Verification

`scripts/check.sh` green and unweakened. **468 tests**, up from 456; 12 new
in `tests/fewrs.rs` — three over the output contract at the process level,
nine over the classification envelope. `scarp doctor` clean.
`git diff --check` clean. Nothing pushed. No recording content in any
artifact; the run output stays under gitignored `.witnessglass/`.

**One observational rerun, and the reason it was not avoidable.** The
envelope is a gate on what the *runner* reports about itself, and no unit
test can prove the runner's own `RunDescriptor` satisfies it — a
mismatched specimen prefix or pair label would leave the frozen assay
permanently `DIAGNOSTIC` and every unit test would still pass. One pass of
the frozen protocol was run, unchanged, to confirm the gate admits it and
the frozen numbers are untouched. It reproduced task:32 cell for cell:
controls PASS, 17 of 30 certified, `STRONG`, agreement 24/30 and 26/30,
18 360 null searches. The `m = 99` diagnostic was not rerun.
