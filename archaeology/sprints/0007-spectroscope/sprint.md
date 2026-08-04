---
id: spr_01KZ7C21FNH9017Z6QKTVS4RM3
sequence: 7
kind: sprint
status: closed
created: 2026-08-04
closed: 2026-08-04
---

# Spectroscope

## Goal

Make three rounds of numerical experiment legible to somebody who does not know what a Haar
transform or a Matrix Profile is — including, and especially, the part where the Matrix Profile
failed.

One page, one shared time axis, and a fixed order of argument:

> what happened → what lens examined it → what the lens found → what the controls say → where the
> lens failed.

An instrument for inspecting evidence. Not a dashboard, and not a place where a conclusion gets
announced.

## Rationale

sprint:4, sprint:5, and sprint:6 produced numbers that are correct, checked, and almost unreadable.
The strongest result any of them produced — that a Matrix Profile ranked a coincidence of two lone
events above the figure the fixture was built to contain — currently exists as a column of `1/1`
occupancy values in a task Result. Somebody who has not read three sprints' worth of archaeology
cannot see it, and it is the single most useful thing this line of work has found.

That is what earns a visualization now and would not have earned one earlier. There was nothing to
project until there were three lenses and a null to hold them against.

**Ground truth is what makes the demo work.** The synthetic oracle is the hero because its structure
was decided in constants before any detector existed, so a page can show what was planted beside
what was found and let a reader judge the gap. That only stays honest if the planted structure comes
from the fixture's own constants and never from the detectors' output — a visualization that
rediscovered its own annotations would be worthless and would look identical.

**Three epistemic classes, kept apart in the rendering.** Injected ground truth, algorithmic
observation, and interpretation are different kinds of claim, and `CLAUDE.md` §2 says neither gets
silently promoted into another. A page that draws a planted region and a detected region in the same
weight has done exactly that promotion, by presentation alone.

### Scope, and why this does not need a new decision

decision:5 lifted the interface non-goal and `CLAUDE.md` narrows what that currently authorizes to
one thing: a foreground, loopback-only, read-only local viewer over one explicitly supplied
recording, behind a per-launch capability, dying with the process. A lift is not a standing budget.

This sprint stays inside that form. Same process shape, same capability, same loopback binding, same
read-only snapshot taken once, same death with the invocation. What changes is that a second
*perspective* is rendered over the same immutable snapshot, and it happens to be a derived
experimental analysis rather than the ledger. It is not a TUI, not a second application, not a
daemon, and not a new surface on the product CLI.

Two things follow, and both are constraints rather than conveniences:

- **The viewer's security machinery is reused, not reimplemented.** A second HTTP server would be a
  second implementation of the capability check, the headers, the not-found equivalence, and the
  no-logging rule, and `tests/view.rs` would cover only one of them. `Viewer` gains a small generic
  way to carry an extra document; it learns nothing about wavelets.
- **The product's `witnessglass view` is unchanged.** Four routes, same page. The experimental page
  is reachable only from an example binary behind the existing experiment feature.

All five of decision:5's conditions still apply and none is relaxed. In particular absences are
still absences: a page whose whole subject is a detector's failure has to render the failure rather
than the plausible thing.

## Success criteria

- One page, over the existing viewer's transport, showing ground truth, the sampled behavioural
  representation, the Haar decomposition against its impulse null, and the Matrix Profile against
  its shuffle null, on a shared and visibly aligned time axis.
- Ground-truth annotations provably sourced from the fixture's generator constants, with a test that
  fails if they ever come from a detector.
- The three epistemic classes distinguished in the rendering by something other than colour alone,
  since colour is decoration everywhere in this codebase by existing convention.
- The Matrix Profile failure visible and explained in place: a top match, its two windows, their
  occupancy, and why a distance of zero was inevitable.
- A reader can select a candidate match and see both matched regions highlighted in the raster.
- No arithmetic in the browser that Rust has not already done. The page renders a derived
  projection; it does not reinterpret raw NDJSON, recompute a transform, or invent a correlation.
- Real recordings work through the same path, with ground truth simply absent rather than faked.
- Deletable: one module, one asset set, one example, one test file, and a generic attachment hook
  small enough to remove with them.

## Non-goals

- Any new detector, any repair of the Matrix Profile representation failure, any changepoint or
  event-native work. sprint:6's recommendation is not started here.
- Reimplementing Haar or Matrix Profile in JavaScript. Rust owns interpretation; this is decision:6
  and it is not negotiable for a page whose whole point is that the mathematics is trustworthy.
- Redesigning the existing workbench, touching unrelated pages, or introducing a visualization
  framework, a charting library, or a general timeline component.
- A statistical significance model. The nulls this project already computed get displayed as they
  are.
- Any change to the raw format, the schema, the recorder, `inspection`, or the product CLI's verbs.
- Committing a real recording, or any wording implying a rendering is safer than the recording
  behind it.

## Outcome

One task, closed. The Spectroscope exists, it runs over the existing viewer's transport, and the
result it renders best is the one three sprints of archaeology buried: a Matrix Profile ranking a
coincidence of two lone events above the figure a fixture was built to contain. On the page you
select the match, two bands land on the raster, and both are almost entirely empty. Nothing has to
be explained for that to land.

### Success criteria, against evidence

- **One page, over the existing transport, with the four tracks aligned.** `Viewer::bind_with` and
  a generic `Attachment` — twenty lines that know nothing about wavelets. `witnessglass view` keeps
  its four routes and its behaviour, and a test asserts an attachment cannot take one of them.
- **Ground truth provably read, not discovered.** Regions come from `oracle`'s constants, and the
  test that proves it feeds a recording with a fixture's session id and completely different
  content, then asserts the annotations do not move while the observed side collapses.
- **Three classes distinguished by more than colour.** Glyph, word, and hue for planted, observed,
  and interpretation, matching the stylesheet's own stated rule.
- **The failure is a first-class exhibit.** Occupancy is displayed per matched window, trivial
  matches are labelled, the explanation sits in place, and the planted figure is offered beside the
  detector's ranking for comparison.
- **Selecting a candidate highlights both regions** across every track at once. It did not work
  when first written and manual inspection is the only reason that was found.
- **No arithmetic in the browser.** Rust computes six aggregations, a Haar decomposition per
  dimension, and eight Matrix Profile ladders before the listener binds.
- **Real recordings work, with ground truth absent rather than faked.**
- **Deletable**: one module, three assets, one example, one test file, and a hook small enough to
  go with them.

### What the sprint found that it was not looking for

**Ranking the profiled dimensions by occupancy excluded exactly the interesting ones.** The
motif-carrying dimensions are sparse by construction — a tool used only inside the figure, a channel
silent everywhere else — so "the eight busiest" threw away the columns the fixture was built to be
interesting in. Preference now comes from the fixture's own constants. It is a small thing that
would have quietly made the hero demo useless.

**Looking at the page found six defects that reading the source did not.** The most serious was a
dead interaction: `decorate()` measured geometry before the stack was in the document, so every
offset was zero and the highlight — the single most important thing on the page — silently did
nothing. The others were a blank row for a dimension whose only event sat on the final bucket, a
label reading "105 of 61 buckets", rows three times taller than their content, the same paragraph
printed three times, and a button offering to highlight two spans of a one-span discord. Five of the
six were invisible in source.

**Two source guards fired on the page's own prose** — the script's comment named an API it promised
not to use, and the not-redacted warning used the word *download*. One was reworded to match the
workbench's convention; the other guard was retargeted at mechanisms so a page can keep warning
about the things it does not do.

### What this sprint deliberately leaves open

sprint:6's recommendation. An event-native motif experiment is still the next detector move, and
changepoint detection is still behind it. Nothing here started either, and nothing here weakens the
case for both.

Whether the Spectroscope should graduate. It is an example behind a non-default feature, and it
should stay one until there is a reason it should not — the standing non-goals are unchanged and
decision:5's lift is still spent on the one viewer, of which this is a second perspective rather
than a second form.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the workbench, or the
product CLI's verbs, and no dependency was added.
