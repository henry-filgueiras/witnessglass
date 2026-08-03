---
id: tsk_01KZ2CFM599MNFZ7HACRBPW2ZY
sequence: 11
kind: task
status: closed
sprint: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
created: 2026-08-02
closed: 2026-08-03
---

# Exercise the viewer against first-contact evidence

## Objective

Point the finished viewer at the original first-contact recording, privately, and find out
whether it materially shortens that investigation while preserving the raw receipts and their
uncertainty.

**Depends on task:8, task:9, and task:10.**

task:4 characterized a 234-record, 580 KB recording with `jq`, in a session that had to
reconstruct the file's shape before it could read it. This task asks whether the viewer answers
the same questions without that, and whether it stays honest while doing so. The measure of
success is not that the interface looks finished; it is that a reader reaches task:4's findings
faster **and** cannot reach a conclusion task:4 refused to draw.

The recording is real and unredacted. It stays local, untracked, unquoted, unexcerpted, and
unchanged, and nothing derived from it enters this repository except aggregate measurements and
interface findings (dragon:2, `CLAUDE.md` §5).

## Acceptance criteria

- Committed fixtures are synthetic and obviously so, and include representative anomalies from
  first contact — an unmatched subagent stop, absent `duration_ms`, absent parentage, a
  duplicated reported description, a truncated tail — reproduced in shape without copying any
  real payload, path, identifier, or excerpt.
- The viewer is run privately against the original first-contact recording.
- The interface can answer, without `jq`:
  - whether the recording is complete;
  - how many tool requests and how many outcomes were recorded;
  - which outcomes are unresolved or anomalous;
  - what activity is attributable to each supplied agent identifier;
  - which subagent boundaries do not pair;
  - whether `duration_ms` and parentage were actually supplied;
  - which raw records support a selected reported/observed correlation.
- The interface is verified **not** to claim a turn count, a causal hierarchy, an execution
  duration, a complete set of file mutations, or the absence of failures and denials. Two silences
  agreeing is not corroboration, and the viewer must not present it as one.
- Startup and interaction behaviour are measured on the real 580 KB specimen and written down as
  numbers. This does not become premature optimization for recordings nobody has.
- Usability and epistemic defects found during the pass are fixed, and the epistemic ones take
  precedence.
- README and user documentation state the final invocation, the security boundary, the privacy
  warning, and the known limitations — what the viewer shows, what it derives, and what it cannot
  know.
- Only aggregate measurements and interface findings are recorded. No line, path, command,
  identifier, or fragment of the recording appears anywhere in the repository.
- The recording is left local, untracked, and byte-for-byte unchanged, and the viewer is confirmed
  to have written nothing.
- `scripts/check.sh` passes, the slice is committed, and dragons 1–3 stay open. If the pass
  produces new evidence about coverage, sensitivity, or identifier meaning, it is appended to the
  dragon it belongs to rather than left here.

## Result

The viewer was run privately against the original first-contact recording. It reproduced every
finding task:4 had extracted by hand with `jq`, and the pass produced one substantial usability
finding that reorganized the workbench before the rest of the task could honestly be called done.

The recording stayed local, untracked, unquoted, unexcerpted, and byte-for-byte unchanged. Its
SHA-256 was taken before the pass and again after, and matched. Nothing derived from it — no line,
path, command, identifier, prompt, payload, fragment, or screenshot — is in this repository, this
result, or the commit message. Every browser-side measurement below was taken by returning
numbers and booleans from the page, never its text.

### 1. The first-use finding

Henry ran the viewer first, on an ordinary happy-path recording. The verdict: epistemically
careful, genuinely informative, and **very busy**.

- The Evidence HUD presented a dozen peer panels ahead of the primary investigation surface.
- Long receipt-chip lists became visual wallpaper.
- All six filter fieldsets were expanded at once.
- The event map turned dense runs into overlapping "caterpillars".
- Panels, tags, identifiers, warnings, aggregates, and controls carried nearly equal weight.
- The map, the filterable ledger, and the inspector — the actual investigative loop — arrived too
  late in the hierarchy.

That is real usability evidence and it is recorded as the reason for what follows. The remedy was
an information-architecture pass, not a reskin: nothing about what the projection derives changed,
and no claim the interface makes was weakened or strengthened.

### 2. The information-architecture change

**Three perspectives, one dominant workflow.** Accessible in-page tabs — click, or arrow keys,
`Home`, `End` — with no routing, no persistence, and no framework.

- **Events (default)** carries the whole loop: a compact summary, the event map, search and
  compact filtering, the canonical ledger, and the inspector as a sticky companion beside it.
- **Coverage** carries evidence gaps, supplied-field coverage, anomalies, per-kind counts with
  honest zero rows and examined scope, subagent boundary pairing, and the standing caveats.
- **Provenance** carries recording identity and schema, recorder-time extrema and their caveat,
  channel counts, capture points, and records by supplied agent identity.

Selection, search, filters, map axis, and inspector state survive perspective switches, in memory
only.

**Completeness did not move.** It leads the compact summary in Events, in alarm styling when a
recording ends mid-record, with the fragment offset and the note that every absence below is
scoped to the valid prefix. Relocating the detailed recording panel to Provenance cannot hide
damage, and the truncated fixture was re-checked specifically for that.

**The summary invents nothing.** It renders each tool-lifecycle kind separately — requested,
succeeded, failed, denied for v2; started and finished for v1 — reading counts Rust already
computed with receipts and examined scope. There is no "outcomes" rollup, because a derived
number without receipts belongs in the Rust projection or nowhere. A guard asserts the browser
computes no such total.

**Progressive disclosure, not receipt deletion.** A receipt list longer than eight collapses to
"N supporting records" and builds its buttons on first open; short lists, pairs, and anomaly
receipts stay inline. The count is always visible and every sequence is one click away. On the
real specimen this took the initially-rendered receipt buttons from several hundred to **20**,
across 10 collapsed sets.

**Filters became a control rather than a wall.** Search always visible; the payload opt-in and
its sensitivity note at the point of use; a Filters disclosure with an active-filter count badge;
removable chips per active filter plus "clear all"; and the anomaly-only shortcut always visible.
The distinction between metadata search and payload search is untouched.

**The map got quiet.** Still exactly one point per record, still recorder order by default, still
no binning, aggregation, span, or duration. The resting mark is a 7px dot inside a 15px hit
target; channel is carried by **shape** — circle, diamond, square — as well as colour and the
labelled legend; anomalous marks are ringed and the selected mark grows. The map takes one tab
stop with a roving tabindex, and arrow keys move between marks.

**The ledger lost two columns.** Sequence, channel, kind, tool, agent attribution, anomaly state.
Correlation ids and secondary fields moved to the inspector rather than holding permanent width.
No payload preview was added.

**Visually calmer.** Fewer equally weighted boxes, hierarchy through spacing and type, monospace
reserved for evidence tokens and identifiers, and a sensitive-recording strip that still says
"Not redacted" and "Rendering is not redacting" without opening anything, with the full
explanation behind a native disclosure.

### 3. Measurements from the real specimen

580,484 bytes, 234 records, schema v2, structurally complete. Release build, macOS.

**Startup**, launch to the URL being printed — replay, validate, project, serialize, bind — over
five runs: **30, 27, 27, 28, 27 ms**.

**Projection over the wire**: 1,356,333 bytes, **2.34×** the recording. Fetched by the browser in
**1.5 ms** over loopback.

**Browser**: `DOMContentLoaded` **21.4 ms**, load event **46.1 ms**, **3,269** DOM nodes for 234
ledger rows and 234 map marks across 2 lanes.

**Interaction**, all measured on the 234-record specimen:

| action | ms |
| --- | --- |
| first selection (builds the inspector cold) | 25.9 |
| subsequent selection | 1.7 |
| anomaly-only filter on | 4.7 |
| filter off | 2.7 |
| metadata search | 2.3 |
| payload search over every record | 3.4 |
| perspective switch | 0.0–0.1 |
| expand the largest receipt set (234 receipts) | 0.3 |
| map axis switch | 2.6 |

Nothing here argues for optimization, and none was done. These are numbers for one 580 KB
specimen on one machine, recorded because task:11 asked for them, and they are not a claim about
recordings nobody has.

### 4. What the interface answered, without `jq`

Every question task:11 listed, read off the interface and cross-checked against the projection:

- **Complete?** Yes — the summary's first figure.
- **Requests and outcomes?** 82 `tool_requested`, 82 `tool_succeeded`, counted separately.
- **Unresolved or anomalous outcomes?** All 82 correlation groups classify as `paired_lifecycle`;
  exactly one anomaly exists in the whole recording, a `subagent_stop_without_start`.
- **Activity per supplied agent identity?** One supplied identity with its records, and 153
  records explicitly unattributed. 234 − 153 = 81 attributable to the child, matching task:4's 81.
- **Subagent boundaries that do not pair?** One start, two stops, the unpaired one named with its
  receipt.
- **Was `duration_ms` supplied?** Never — 0 of 82. **Parentage?** Never — 0 of 3.
- **Which raw records support a correlation?** Receipts, on every derived claim.
- 65 `reported_intent` records, matching task:4 exactly; `prompt_id` on 233 of 234.

Every figure agrees with the hand analysis in task:4. That is the strongest evidence available
that the projection is honest: two independent methods, one with `jq` and one with this
interface, over the same file, reaching the same numbers.

### 5. What it was verified *not* to claim

Checked on the real specimen by testing the rendered text of all three perspectives for
forbidden readings, returning booleans only:

- no turn count, and no `prompt_id` grouping;
- no "still running" anywhere — an unresolved request reads "outcome not observed";
- no root agent; the only occurrence of the phrase is the explicit denial "An absent identity is
  not supplied — it is not a root agent";
- no execution duration and no elapsed-time claim;
- no "files changed" list;
- no "no failures occurred". Zero `tool_failed` reads "no `tool_failed` record observed" with its
  examined scope, and the standing note that two silences agreeing is not corroboration is
  present, as are the unexercised-surface and filesystem-effect caveats.

### 6. Verification performed

In a real browser, against the synthetic fixtures: Events is the initial perspective; the loop is
comprehensible without traversing a detailed HUD; tab controls work by keyboard with a roving
tabindex and communicate selection; selection, search, filters, and row set survive two
perspective switches; a 234-receipt set is collapsed initially and expands to 234 clickable
receipts, each of which returns to Events with that record selected; filtering fades 28 of 34 map
marks and leaves canonical order untouched; the map has exactly one tab stop and arrows move
selection between marks; payload search retains its warning and opt-in; the hostile-payload
fixture yields **zero** injected `img`, `svg`, `[onerror]`, or `[onload]` elements while the
payload is visible as text; and the truncated fixture shows damage in the Events summary and
scopes `session_ended`'s absence to the valid prefix.

Defects found by looking, and fixed: the Filters badge rendered "0" when nothing was filtered,
because a class `display` beats the `hidden` attribute; the legend's swatches carried colour but
not shape; and `--faint` sat at a 4.17:1 contrast ratio in dark mode, below WCAG AA, raised to
5.13:1 with the light equivalent raised to 5.28:1. Every semantic colour now clears AA against
its background in both palettes, measured rather than assumed.

Gate: `./scripts/check.sh` passes. 168 tests
(0, 0, 11, 18, 27, 2, 18, 35, 9, 8, 7, 19, 14), `scarp doctor: 25 artifact(s) checked, no
problems found`. Four new source-level guards cover the perspectives-as-real-tabs contract,
collapsed-but-reachable receipts, the no-rollup rule, and one-point-per-record.

### 7. Remaining limitations

- **The harness could not resize the browser out of fullscreen.** The narrow layout was verified
  by opening a second window at a real 1024px viewport and reading its computed layout — the split
  collapses to one column, the inspector loses its left border and gains a top one, and the body
  does not overflow. That is a real measurement at a real viewport, but it is not the same as
  looking at it, and it stays on the checklist.
- **The light theme could not be observed**, because the OS colour scheme cannot be emulated here.
  Its palette was verified by computing WCAG contrast ratios from the parsed stylesheet. Also
  still on the checklist.
- **The specimen is still one clean happy-path recording.** Failure, denial, interruption, resume,
  and multi-turn rendering are exercised only by synthetic fixtures. The interface renders them;
  no real data has ever tested that it renders them *correctly*. A deliberately hostile recording
  remains the highest-value next measurement, and would also answer dragon:3's first-order
  question.
- **The projection serializes each raw record twice**, once in `records` and once inside its
  ledger entry, which is where the 2.34× ratio comes from. At 1.36 MB over loopback this costs
  1.5 ms and nothing else, so it was left alone rather than optimized on speculation. If a future
  specimen makes it matter, the fix is a sequence reference rather than a copy.
- **Guards are not browser tests** and are not described as one anywhere. Visual and interaction
  correctness depends on the checklist being run by a human.

### Scarp desire paths

**idea:1 recurred, for the eleventh time.** Result written to a temporary file and appended with
a shell redirect before `scarp close task:11`. Eleven for eleven, and this is the last task of the
sprint, so the sample is closed at eleven.

**No new idea is filed.** Scarp handled a task close and a sprint close without incident. The
friction in this task was entirely in a browser that would not resize, which is not a Scarp
problem.
