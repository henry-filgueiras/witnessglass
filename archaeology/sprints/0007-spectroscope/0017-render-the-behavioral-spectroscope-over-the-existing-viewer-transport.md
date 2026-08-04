---
id: tsk_01KZ7C2XGGWH0ABV92VQFJTW8W
sequence: 17
kind: task
status: closed
sprint: spr_01KZ7C21FNH9017Z6QKTVS4RM3
created: 2026-08-04
closed: 2026-08-04
---

# Render the behavioral spectroscope over the existing viewer transport

## Objective

Build the Behavioral Spectroscope: one experimental page, over the existing viewer's transport, that
makes sprint:4's substrate, sprint:5's Haar decomposition, and sprint:6's Matrix Profile — including
its failure — legible without prior knowledge of either transform.

The page argues in a fixed order and the order is the deliverable: what happened, what lens looked
at it, what the lens found, what the control says, and where the lens broke. A reader who follows it
should be able to point at the synthetic oracle and say what was planted, how sparse the sampling
is, what Haar saw across scales, how that differs from an isolated impulse, what the Matrix Profile
called a motif, why some perfect matches are coincidences, and where the planted recurrence actually
lives.

## Acceptance criteria

- **One derived projection, computed in Rust, rendered by the browser.** A `spectroscope` module
  assembles ground truth, the sampled signal at several display scales, the Haar decomposition with
  its impulse null, and the Matrix Profile ladder with its shuffle null, into one serializable
  document. The page performs no transform, no distance, no re-bucketing, and no parse of raw
  NDJSON. decision:6 already settled where meaning lives and this does not move it.
- **Ground truth comes from the fixture's generator constants, never from a detector.** The
  document carries injected regions only when the recording's session id matches a known synthetic
  fixture, and a test asserts the annotations equal the `oracle` constants rather than anything
  derived. A real recording gets no ground truth at all — absent, not empty, not inferred.
- **Three epistemic classes are distinguishable in the rendering**, and by more than colour: planted
  truth, algorithmic observation, and interpretation each carry a word and a shape as well as a
  hue, matching the existing stylesheet's stated rule that colour is decoration everywhere.
- **A shared, aligned time axis** across the ground-truth band, the behavioural raster, the Haar
  scalogram, and the Matrix Profile curve, close enough that a reader can trace a moment vertically
  through all four.
- **The raster shows sparsity as sparsity.** Occupied buckets are visible, empty ones are visibly
  empty, magnitude is distinguishable from presence, and nineteen dimensions do not become an
  unreadable wall — active dimensions first, the rest available on request, full labels recoverable.
- **A display-scale control** re-expresses the raster at coarser aggregations, computed in Rust from
  the same machinery, and says plainly that the chosen aggregation is a display choice rather than
  the canonical representation or an optimum any transform derived.
- **The Haar view is a scalogram with a null toggle.** Time horizontally, dyadic scale vertically
  labelled in human units, intensity from the detail coefficients the experiment already computes,
  and a switch between observed magnitude and magnitude read against the isolated-impulse null. The
  8 s motif's signature is described as what sprint:5 measured — energy through the period and a
  collapse above it — and never as a spectral peak.
- **The Matrix Profile view exposes the committed ladder**, the distance curve for a selected
  dimension and window, the top candidate pair, a discord, and the shuffle-null comparison per
  window. Selecting a candidate highlights both matched windows in the raster, so the reader judges
  the match instead of trusting it.
- **The failure is a first-class exhibit.** For a top trivial match the page shows both windows'
  occupancy and states, in place, that two windows each holding one non-empty bucket at the same
  offset are identical after subsequence normalization, so a distance of zero was arithmetic rather
  than evidence. Where the fixture allows it, that match is contrasted against the planted
  recurrence.
- **The narrative is generated from fixture constants and committed experiment outputs**, with every
  number computed rather than written, and no language model anywhere in the path.
- **Real recordings work through the same entry point**, with conservative labels — candidate
  recurrence, candidate discord — and no ground truth.
- **The transport is the existing viewer's.** No second server, no second capability implementation,
  no new headers, no filesystem-mapped route. `witnessglass view` keeps its four routes and its
  behaviour.
- Tests for the projection, for the ground-truth provenance, for index-to-time conversion, and
  source-level guards over the new assets matching the existing workbench's. `scripts/check.sh`
  passes unweakened; no existing test is changed.
- Manual inspection of the rendered page, with the obvious defects fixed and the remaining
  compromises recorded.

## Result

Delivered. The Behavioral Spectroscope renders sprint:4's substrate, sprint:5's Haar decomposition,
and sprint:6's Matrix Profile — including its failure — on one shared time axis, over the existing
viewer's transport, with the three kinds of claim kept visibly apart.

The thing that works best is the thing the round was for: on the legible oracle you can select the
Matrix Profile's rank-one match, watch two bands land on the raster, and see that both of them are
almost entirely empty. The failure is not described, it is visible.

### 1. Architecture: the viewer's transport, one generic hook

`Viewer` gained `bind_with(snapshot, attachments)` and an `Attachment` — a `'static` route, a
content type, and a body. Twenty lines, no knowledge of wavelets, no feature gate. `bind` is now
`bind_with(snapshot, vec![])`, so `witnessglass view` is byte-identical in behaviour and keeps its
four routes.

A second HTTP server was the alternative and was rejected: it would have been a second
implementation of the capability check, the security headers, the not-found equivalence, and the
no-logging rule, with `tests/view.rs` covering only one of them. Attaching a document reuses all of
it — authorization is still decided before routing, so an attachment cannot be read without the
capability, and a test asserts an attachment cannot take a built-in route or a relative one.

Everything else lives in `src/experiment/`: a `spectroscope` module that assembles the document, its
three assets, and `examples/spectroscope.rs` which serves them. The page is reachable only from that
example, behind the existing `experiment-matrix-profile` feature.

**The browser does no mathematics.** `spectroscope::project` runs the signal projection at six
display aggregations, a Haar decomposition per dimension, and a Matrix Profile ladder for eight
dimensions, and serializes the result before the listener binds. The script maps milliseconds onto
x coordinates and magnitudes onto opacities, and that is the whole of its arithmetic. decision:6 is
where this comes from and it did not need bending.

Document size for the legible oracle: 468 KB, one same-origin fetch.

### 2. Three classes of claim, and why ground truth cannot be discovered

`Class` tags every block as `planted`, `observed`, or `interpretation`. In the rendering each
carries a glyph (■ ▲ ●), the word, and a hue — never the hue alone, matching the stylesheet's own
stated rule that a reader who cannot see the colour must still be able to tell a generated fact from
a measured one.

`GroundTruth` is populated only when the recording's session id matches a known fixture, and every
region is read from `oracle`'s generator constants. **The test that matters** feeds a recording
carrying the legible oracle's session id and nothing else of it — three records, one unrelated tool,
a four-second span instead of four minutes — and asserts the regions are byte-identical to the real
fixture's while the observed side collapses. A page that reverse-engineered its annotations would
look the same and be worthless; this is the difference, checked.

Regions are also asserted to tile the recording contiguously, so the band cannot imply quiet the
generator did not leave.

### 3. What shipped

**Four perspectives**, as real ARIA tabs with arrow-key navigation:

- **Overview** — planted band, the six busiest dimensions, five Haar scales, one Matrix Profile
  curve, and the five-step narrative. Section rules name the transitions: *what happened*, *Haar,
  across scales*, *Matrix Profile, one window*.
- **Haar** — the full scalogram for a selected dimension, a per-level table of share against the
  isolated-impulse share with the ratio, and a *fade what the null explains* filter.
- **Matrix Profile** — the committed ladder as buttons, the distance curve with gaps where
  subsequences were excluded, ranked candidates, a candidate discord, the per-window null table, and
  the planted figure offered beside them for comparison.
- **Raw** — every active dimension, a six-stop display-aggregation scrubber, and a toggle for
  dimensions with no activity.

**Interactions**: a shared cursor with a time readout tracking across every track in a stack;
click-to-highlight on any candidate, painting both spans across the planted band, the raster, and
the profile curve at once; dimension selection; window selection; display aggregation; the null
filter.

**Progressive disclosure**: the Overview shows six dimensions of nineteen, Raw shows the active ones
with the rest a checkbox away, and long labels are shortened in the gutter with the full label on
the row's tooltip.

### 4. The five-step narrative, generated

Built in Rust from fixture constants and computed values. No language model anywhere in the path.
On the legible oracle it reads, in full:

> **■ We planted this.** This is the legible oracle — deliberately dense, best case. Its generator
> placed 5 regions, including a figure repeating exactly every 8 s and a sustained block of
> different character. The bands above come from those constants — nothing on this page discovered
> them.
>
> **▲ The sampled signal is mostly empty.** At 500 ms, 376 of 481 buckets hold no record — 78.2%
> empty. That emptiness is the single biggest influence on everything below, and it is a property of
> sampling an event stream rather than of the session.
>
> **▲ Haar saw this.** […] The largest here is recorded_response_json_bytes at the 64 s scale,
> carrying 5.8× what isolated events alone would give.
>
> **▲ Matrix Profile saw this.** Its strongest match anywhere is in records at a 2 s window: two
> spans at a distance of 0.000, holding 1 and 1 non-empty buckets respectively.
>
> **● And that match is arithmetic, not evidence.** […] After subsequence normalization they are
> identical, so a distance of zero was inevitable regardless of what surrounds them.
>
> **● Therefore.** Haar recovered usable scale structure […] Sampled univariate Matrix Profile did
> not recover the planted figure reliably.

Every number is measured. The closing step differs when there is no fixture, because without a
planted figure there is nothing to have missed and saying otherwise would be the promotion this page
exists to prevent.

### 5. Manual inspection, and the six defects it found

The page was served and driven in a browser at 1440×1000, across all four perspectives, with the
console watched. Six things were wrong and five of them were only findable by looking:

1. **Track rows were ~72 px tall** for a 10 px drawing, because a three-line label forced the grid
   row. The Overview needed four screens of scrolling. Labels are now two compact lines and rows are
   ~48 px.
2. **`decorate()` measured geometry before the stack was in the DOM**, so `offsetLeft` and
   `offsetWidth` were zero and *the highlight never appeared* — the single most important
   interaction on the page, silently dead. Now decorated after insertion.
3. **The final bucket was drawn outside the viewBox.** A dimension whose only record is the closing
   `session_ended` rendered as a completely blank row while its label said "1 of 481 buckets". The x
   is now clamped so the last bucket stays visible.
4. **"105 of 61 buckets"** — occupancy was read from the base scale and the total from the displayed
   one. Both now come from the row being drawn.
5. **The same paragraph three times** under three trivial matches. Explained once, referenced after.
6. **"highlight both spans" on a discord**, which has one span. The label now counts.

Two source-level guards also fired on my own prose — the script's comment used the name of an API it
was promising not to use, and the page's not-redacted warning used the word *download*. The comment
was reworded to match the workbench's convention of never writing those names; the export guard was
retargeted at mechanisms (`download=`, `blob:`, `createObjectURL`, `navigator.clipboard`) so the
page stays free to warn about the things it does not do.

No console errors, no CSP violations, no network requests beyond the one same-origin fetch.

What the inspection confirmed, on the Haar tab, is worth recording because it is the clearest thing
on the page: at 1 s, 2 s, and 4 s the motif shows as four discrete pulses in each motif region; at
8 s they merge into a contiguous block at ×1.39 of the null; **at 16 s the row nearly vanishes, at
×0.35**. sprint:5's cliff, drawn.

### 6. Real-recording mode shipped

The same entry point, no ceremony. Verified against the untracked real session: `ground_truth` is
`None`, no planted band renders, the narrative opens with *No ground truth here — this recording is
not a synthetic fixture, so nothing on this page knows what it contains*, and every finding is
labelled a candidate. Nothing from sprint:6's manual interpretation is hard-coded anywhere.

No real recording was committed, copied, or reproduced.

### 7. Compromises, recorded rather than hidden

- **Eight dimensions get a Matrix Profile, not nineteen.** Six windows and a shuffled null each is a
  real cost, and nineteen profiled dimensions is more than the page can show honestly. The
  unprofiled ones are listed in the document so the omission is visible, and a test asserts profiled
  plus unprofiled equals all.
- **Ranking those eight by occupancy alone was wrong and had to be fixed.** The motif-carrying
  dimensions are sparse *by construction* — a tool used only inside the figure, a channel silent
  everywhere else — so the densest-eight rule excluded exactly the columns the fixture was built to
  be interesting in. Preference now comes from the fixture's own constants, which cannot become a
  way of discovering which dimension looks good.
- **The scalogram normalizes each row to its own maximum**, so a row shows where its energy is
  rather than how it compares to another row's. Cross-row comparison is the null table's job, and
  the two are adjacent for that reason.
- **The whole document is computed up front**, so a large recording costs a large payload and a
  slow start. Fine for session-sized recordings; not a scale claim.
- **The null filter fades rows rather than transforming them.** It is a filter, not a second
  statistic, and no significance model was invented.

### 8. Desire-path friction

**Genuinely low this round, and the reason is worth recording.** Both artifacts were minted with
`scarp new sprint --body-file` and `scarp new task --sprint sprint:7 --body-file` on the first
attempt: the section-template refusal that hit three consecutive rounds did not recur, because this
task's material fitted `Objective` and `Acceptance criteria` without wanting a third section. The
friction was never about the command; it was about rounds whose content did not match the template,
and this one did.

**Appending a Result and an Outcome is still `cat >>` on this machine**, which is scarp 0.2.0 —
`scarp close` offers only `--resolved-by`. maintenance:1 records that upstream shipped
result-on-close; this repository has not picked it up. Noted as a version lag rather than as a gap,
which is the correction task:16's addendum already had to make.

**idea:5 is untouched by this round** and still stands: nothing here needed a sealed section,
because nothing here was a prediction.

### What this task did not do

No detector. No repair of the Matrix Profile representation failure — it is rendered, not fixed. No
changepoint work, no event-native work, no multivariate fusion, no significance model. No change to
the raw format, the schema, the recorder, `inspection`, or the product CLI's verbs. No redesign of
the workbench and no edit to any of its assets. No new dependency. No charting library and no
timeline framework: every mark on the page is an SVG rect or path drawn from the document.

Nothing pushed. sprint:5's and sprint:6's conclusions are unedited.
