---
id: tsk_01KZ2CEZ9GS0ZYR32VANEA75SD
sequence: 10
kind: task
status: closed
sprint: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
created: 2026-08-02
closed: 2026-08-02
---

# Build the first local evidence workbench

## Objective

Build the browser workbench: four synchronized surfaces over one projected recording, in which
reported, observed, and derived information stay visibly distinct and every derived claim leads
back to the records that support it.

**Depends on task:8 and task:9.** The browser renders what the projection derived and what the
loopback boundary served. It does not parse NDJSON, redefine lifecycle semantics, or invent a
correlation of its own.

This is the surface where decision:5's argument against itself is tested. Raw NDJSON is hostile
enough that nobody mistakes it for a complete account of a session; a clean interface is not. The
interface's job is to make first contact's findings legible **and** to make its blind spots
impossible to miss.

## Acceptance criteria

Four synchronized surfaces:

**1. Evidence HUD**

- Schema version, completeness, record count, session identifier, recorder time range, channels,
  event kinds, mechanisms, and supplied agent identifiers.
- A conspicuous sensitive-recording warning. Nothing anywhere implies the rendered view is safer
  to share than the recording behind it (dragon:2).
- Session-specific evidence gaps shown just as conspicuously — absent durations, absent parent
  identifiers, and any unexercised surface.

**2. Event map**

- Point events only. No fabricated execution bars.
- Recorder order by default.
- An optional recorder-time projection, labelled derived, which never reorders the ledger.
- Lanes only for identities actually supplied, plus an explicitly unattributed lane. "Not
  supplied" is never rendered as "root".
- Unmatched and anomalous boundaries visible rather than quietly dropped.

**3. Event ledger**

- Canonical append sequence.
- Filters for channel, kind, tool, mechanism, supplied agent identity, and anomaly state.
- Metadata search by default. Any payload-inclusive search requires an explicit opt-in and makes
  its sensitivity clear at the point of use.

**4. Evidence inspector**

- The full record envelope, provenance, context, event data, and semantically rendered JSON.
- Correlated request, reported intent, and outcome shown as **separate** evidence, never merged
  into one step.
- Every derived label linked to the sequence numbers supporting it.
- Payloads collapsed until explicitly revealed.

Across all four:

- Reported, observed, and derived are distinguishable by label and structure. Colour is never the
  only distinction.
- Missing evidence is phrased as the sprint requires: "no failure record observed", "outcome not
  observed", "agent identity not supplied", "stop without observed start".
- Keyboard navigation, visible focus, sufficient contrast, and reduced-motion behaviour.
- Recording text is never rendered through `innerHTML`, Markdown, automatic linkification, a
  template that bypasses escaping, or any equivalent unsafe path. A recording contains commands
  and file contents chosen by nobody trustworthy.
- The browser layer is small and bundled. No package-manager or build-tool ecosystem is adopted
  unless it materially improves correctness, and its cost is justified in the result.
- An honest smoke-test strategy. If full headless-browser infrastructure would dwarf the viewer,
  say so, document the tradeoff, and keep a deterministic manual smoke checklist. Static string
  assertions over bundled assets are not UI testing and are not described as such.
- Fixtures remain synthetic; no real recording is opened by this task.
- `scripts/check.sh` passes, the slice is committed, and dragons 1–3 stay open.

## Result

The workbench exists: four synchronized surfaces over one projection, served by task:9's process
and rendering only what task:8 derived. `src/assets/viewer.{html,css,js}`, compiled into the
binary, plus one new route and one CSP change in `src/view.rs`. **No package manager, bundler,
transpiler, framework, or test runner was added**, and no Rust dependency either.

`docs/viewer.md` is the user-facing document: what it shows, what it derives, what it cannot
know, the security boundary, and a 28-point manual smoke checklist.

### 1. The four surfaces

**Evidence HUD.** Recording identity and completeness; recorder time range with each extremum
citing its sequence, and non-monotonic records called out as descriptive rather than damaged;
per-channel counts; per-kind counts across the schema's **whole** vocabulary so a kind with no
records is a first-class row; capture points; records per supplied agent identity with an
explicit "identity not supplied" row; subagent boundaries; an evidence-gaps panel; and every
anomaly with receipts.

**Event map.** One mark per record, point events only, no execution bar anywhere. Lanes for each
supplied `context.agent_id` plus one lane labelled "identity not supplied" — never "root". Axis
is append sequence by default; the recorder-time axis is a labelled radio with a `⟐ derived` tag
and a note saying spacing is not execution duration. Switching it does not touch the ledger, and
a test asserts that.

**Event ledger.** Canonical append order, always, with the count line saying so even when
filtered. Filters for channel, kind, tool, mechanism, supplied agent, and anomaly state. Search
is metadata-only by default; payload search is a separate checkbox whose note says at the point
of use that it reads commands, file contents, and tool output.

**Evidence inspector.** Envelope and provenance; a separate "lifecycle subject" section for
subagent events so a child id can never be read as the emitter; anomalies citing the record;
correlated evidence; and the raw record behind a collapsed `<details>`.

Selection is synchronized across all three of map, ledger, and inspector, in both directions.
Every receipt is a button that selects its record.

### 2. Where the epistemics live in the rendering

- **Reported, observed, and derived carry a glyph *and* a word** — `◇ reported`, `● observed`,
  `▣ recorder`, `⟐ derived`. Colour is decoration; remove it and every distinction survives.
- **Correlated evidence is placed beside, never merged.** Selecting a request shows separate
  "Reported" and "Observed" groups, each card carrying its own receipt and an honest role label:
  "a claim", "request — not proof of execution", "outcome: executed successfully", "outcome:
  denied, did not execute". The conflicting-outcome group renders as three separate cards under
  the shape "ambiguous — nothing was paired", with neither outcome chosen.
- **A paired lifecycle renders as `#2 → #4`**, captioned "Two positions in the append chain. Not
  elapsed time, not execution duration, not nesting, not containment."
- **Absences use the sprint's own words**: "outcome not observed", "agent identity not supplied",
  "stop without observed start", "no `tool_failed` record observed", "parent identity not supplied
  — none is inferred from containment, adjacency, or timing". A test asserts each phrase is
  present and that "still running", "root agent", and "turns" are absent.
- **Every zero carries its scope.** On the truncated fixture, `session_ended` reads "no
  session_ended record observed" with "examined: 33 record(s) — the valid prefix of a recording
  that stops mid-record". On a complete one the same claim reads against all records. That
  distinction survives from `ExaminedScope` to the pixel, which is what task:8 was for.
- **The evidence-gaps panel volunteers two things nobody asked it for**: that an unexercised
  surface is not a working surface and two silences agreeing is not corroboration, and that tool
  events do not reveal every file a session changed.
- **`prompt_id` renders on its record with "It groups nothing: what it delimits is
  unestablished."** It is not a filter, not a lane, and not a segment.

### 3. Security

`script-src` moved from `'none'` to `'self'` — the minimum the workbench needs, since the script
is served from this origin and there is no inline script, no `eval`, and no nonce. Everything
else is unchanged and `unsafe-inline`/`unsafe-eval` appear nowhere; a test asserts that.

No recording text reaches an HTML-parsing sink. `el()` sets `textContent` and attributes and
nothing else, and `tests/workbench.rs` asserts by name that no markup-assigning property,
adjacent-markup insertion, document write, range fragment parser, or `eval` appears in the file.
Verified in a browser against a payload containing `</script><img src=x onerror=alert(1)>`: zero
`img`, `svg`, `[onerror]`, or inline `script` elements exist on the page, and the payload is
visible as text.

Nothing is persisted — no storage, cookie, worker, or cache — asserted both by source guard and
by reading `localStorage.length`, `sessionStorage.length`, `document.cookie`, and the service
worker controller in a live browser, all empty. The capability is read once from `location.search`
into one constant, used in one `fetch`, and written nowhere; no `history.pushState` or
`replaceState`.

### 4. On not adopting a frontend toolchain

The browser layer needs the DOM, `fetch`, and `URLSearchParams`. A framework would supply
reactive rendering that four surfaces over an **immutable** projection do not need — a render is a
pure function of state that changes only on selection, filter, or search. A bundler would supply
module resolution for zero dependencies and minification for a file fetched over loopback. Both
would add a `node_modules` tree and a lockfile to a repository whose claim is that you can read
what it does, and both would insert a build step between the source and the bytes served — in a
tool whose whole value is that the rendering is not a second opinion.

The cost is accepted and stated in `docs/viewer.md` §5: no type checking on the JavaScript, and no
framework-supplied escaping discipline. The second is answered by making `textContent` the only
path content takes and asserting the absence of every sink by name.

### 5. Testing, honestly

`tests/workbench.rs` holds two different things and says which is which in its own module
documentation.

**Fixture tests (real).** `fixtures/synthetic-first-light.ndjson` — 34 records, hand-built, every
line self-evidently synthetic — replays and projects through the same code the viewer uses, and
each shape it exists to demonstrate is asserted present: a request with no outcome, a denial with
no request, duplicate requests, divergent tool names, conflicting outcomes, an unmatched subagent
stop, `duration_ms` on 1 of 12, parentage on 0 of 3, a duplicated reported description, a
subagent's work attributable by `context.agent_id` bracketed by an `Agent` call's interval with no
parentage derived, a clock that moves backwards, and a markup-shaped payload. Its truncated
companion asserts the valid-prefix scope reaches the projection. Both load through
`Snapshot::load`, the viewer's own entry point.

**Source-level guards (not UI testing, and not described as such).** They read source text and
cannot tell you the page renders, that a mark lands where it belongs, or that focus survives a
keypress.

**Headless-browser infrastructure was considered and declined.** A driver, a browser binary in CI,
and the flake management that comes with them would be several times the size of the thing under
test, for a page served to one browser on one machine. The trade is that visual and interaction
correctness is verified by hand against the 28-point checklist in `docs/viewer.md` §7 — and that
the checklist has to actually be run.

**It was run, in a real browser, and it found two defects that no guard could have.**

- **Every map mark was stacked at position zero.** `style-src 'self'` refuses inline `style`
  attributes, so `setAttribute("style", "left:47%")` was silently dropped by the browser and every
  record sat on top of every other. Fixed by setting positions through the CSSOM, which CSP does
  not restrict — the alternative was widening the policy to `'unsafe-inline'` to move a dot.
- **Arrow-key navigation stopped after one step.** Selecting re-rendered the whole ledger, which
  destroyed the focused row and sent focus to the document. Fixed by separating selection from
  structure: filters and search rebuild, selection only toggles classes. This was both an
  accessibility defect and a needless rebuild per click.

Two smaller layout defects were also found and fixed by looking: notes wrapping one character per
line, and the ledger and inspector both too narrow side by side below 1200px.

Gate, final run:

```
==> shell syntax / cargo fmt / cargo clippy / cargo test
    0, 0, 11, 18, 27, 2, 18, 35, 9, 8, 7, 19, 10 passed; 0 failed
==> scarp doctor
doctor: 25 artifact(s) checked, no problems found
==> all checks passed
```

164 tests, up from 154. Three existing `tests/view.rs` tests failed when the page gained a script
and the CSP widened; they were updated to assert the new reality rather than relaxed — the CSP
test now also asserts `unsafe-inline` and `unsafe-eval` appear nowhere.

### 6. What changed outside the task

`README.md` lists the workbench under what works, points at `docs/viewer.md`, names the two
committed fixtures so the viewer can be tried without a real recording, and says plainly that the
viewer **has not yet been run against a real recording**. `docs/viewer.md` is new.

No real recording was opened, listed, or referenced by this task. Both fixtures are synthetic and
a test asserts every line says so.

### Scarp desire paths

**idea:1 recurred, for the tenth time.** Temp file, shell redirect, then `scarp close task:10`.
Ten for ten.

**No new idea is filed.** Scarp was not involved in this task beyond opening and closing it.

### What task:11 inherits

Everything it needs, and one thing it has to decide. The fixtures already satisfy its first
acceptance criterion — representative anomalies from first contact reproduced in shape without
copying any real payload, path, identifier, or excerpt — so its remaining work is the private
pass against the real recording, the startup and interaction numbers on the 580 KB specimen, and
whatever the pass finds.

Two concrete things to expect on the real specimen:

- The serialized projection is about **2.6×** the recording's size, measured on synthetic bulk
  data, because each raw record is serialized twice: once in `records` and once inside its ledger
  entry. A 580 KB recording should produce roughly 1.5 MB of JSON. That is fine to fetch over
  loopback and is worth collapsing to a sequence reference if task:11's measurements say it costs
  anything real.
- Every count panel renders **all** receipts, capped at 24 per row. On 234 records that is fine;
  the caps are stated in the interface rather than silently applied.
