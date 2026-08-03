# The local evidence workbench

```sh
witnessglass view --recording <PATH>            # opens a browser
witnessglass view --recording <PATH> --no-open  # prints the URL and waits
```

One recording, replayed and projected before anything is bound, held as a single immutable
in-memory snapshot, and served read-only to a browser on an operating-system-selected port on
`127.0.0.1` behind an unguessable per-launch capability. Foreground and short-lived. It dies
with the command; Ctrl-C ends it and leaves no listener and no state behind.

**A rendered recording is exactly as sensitive as the recording.** Rendering is not redacting.
There is no export, no download, and no share affordance, and there will not be one until a
capture and redaction contract exists (dragon:2).

## 1. What it shows

Four synchronized surfaces over one projection.

**Evidence HUD.** Schema version, completeness, record count, session identifier, recorder time
range, per-channel counts, per-kind counts across the whole schema vocabulary, capture points,
records per supplied agent identity, subagent boundaries, an evidence-gaps panel, and every
anomaly with its receipts.

**Event map.** One mark per record, in lanes. Point events only. Default axis is append
sequence; an optional recorder-time axis is labelled derived and never reorders the ledger.
Lanes exist for supplied agent identities and for an explicitly unattributed lane; anomalous
records carry a ring and a glyph.

**Event ledger.** Canonical append order, always. Filters for channel, kind, tool, mechanism,
supplied agent identity, and anomaly state. Search covers metadata by default; payload search
is a separate opt-in that says what it reads at the point of use.

**Evidence inspector.** The selected record's envelope, provenance, context, and payload —
collapsed until revealed — plus every other record sharing its correlation id, shown as
separate cards with their own receipts.

Every derived claim in the interface carries the raw sequence numbers supporting it, as
clickable receipts. Every zero carries the scope it was counted in.

## 2. What it derives, and what derived means here

Rust derives everything; the browser renders it. The page does not parse NDJSON, does not
redefine lifecycle semantics, and does not invent a correlation the projection did not license
(decision:6).

Derived material is marked `⟐ derived` and is limited to: correlation groups keyed by the
schema's own identifier, a cardinality classification, anomalies, aggregates, field-coverage
summaries, and descriptive timestamp extrema.

Reported, observed, and recorder are **raw provenance channels** and are shown as such. Derived
is not a fourth channel. Each is carried by a glyph *and* a word, so colour is never the only
distinction.

## 3. What it cannot know, and does not claim

These are load-bearing. Each corresponds to something task:4 measured or dragon:3 forbids.

- **No turn count and no `prompt_id` grouping.** `prompt_id` is shown on the record that carried
  it and counted for presence. It defines no group, segment, or unit of work while dragon:3 is
  open.
- **No execution duration.** A one-to-one correlation is rendered as two positions in the append
  chain — `#2 → #4` — explicitly labelled as not elapsed time, not execution duration, not
  nesting, not containment. Where a `duration_ms` was supplied it is shown as supplied; where it
  was not, coverage says so.
- **No causal hierarchy.** Parent identity appears only where a subagent event delivered it.
  Where it did not, the interface says "parent identity not supplied — none is inferred from
  containment, adjacency, or timing". Records falling inside another call's sequence interval are
  not thereby its children, and nothing draws a line between them.
- **No root agent.** An absent `context.agent_id` renders as "agent identity not supplied", in
  its own lane and its own aggregate row. A subagent lifecycle event's `agent_id` names the child
  the event is *about* and is kept in a separate "lifecycle subject" section.
- **No complete account of what a session changed.** Tool events do not reveal filesystem effects
  of a command. The evidence-gaps panel says so, unprompted.
- **No absence of failures or denials.** A kind with no records reads "no `tool_failed` record
  observed", with the examined scope attached. Two silences agreeing is not corroboration, and
  the panel says that too.
- **An unexercised surface is not a working surface** and is not rendered as one.
- **A truncated recording's absences are scoped to its valid prefix**, not to a complete
  recording, everywhere the interface reports one.

## 4. The security boundary

- Loopback only. There is no flag, environment variable, or configuration that binds elsewhere.
- Every route — page, stylesheet, script, projection — requires the capability. An unauthorized
  request gets the same 404 as an unknown path, carrying no session id, record count, schema
  version, or quoted record.
- `Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'; connect-src
  'self'` plus `img-src`, `font-src`, and `object-src` at `'none'`, `frame-ancestors 'none'`,
  `base-uri 'none'`, and `form-action 'none'`. Plus `nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, `no-store`, and the three cross-origin isolation headers.
- No inline script and no inline style attribute. Marks are positioned through the CSSOM, which
  CSP does not restrict, rather than by widening `style-src`.
- Nothing is persisted: no storage, no cookie, no worker, no cache. The capability is read once
  from the URL into one constant and written nowhere.
- No recording text ever reaches an HTML-parsing sink. Everything goes through `textContent`.
- Nothing about the request stream is logged.
- Two assets and one script, compiled into the binary. No CDN, font, telemetry, or third-party
  asset, and no request leaves the machine.

## 5. On not adopting a frontend toolchain

No package manager, bundler, transpiler, framework, or test runner was added. The browser layer
is one HTML file, one stylesheet, and one ES module, all `include_str!`-compiled into the binary.

The workbench needs the DOM, `fetch`, and `URLSearchParams`. A framework would supply reactive
rendering the four surfaces do not need — the projection is immutable, so a render is a pure
function of state that changes only on selection, filter, or search. A bundler would supply
module resolution for zero dependencies and minification for a file the browser fetches over
loopback. Both would add a `node_modules` tree and a lockfile to a repository whose whole claim
is that you can read what it does, and both would put a build step between the source and the
bytes served — in a tool whose entire value is that the rendering is not a second opinion.

The cost is real and is accepted: no type checking on the JavaScript, and no framework-supplied
escaping discipline. The second is answered by making `textContent` the only path content takes
and asserting the absence of every HTML sink by name in `tests/workbench.rs`.

## 6. Testing, and what is not tested

`tests/workbench.rs` holds two different things and says so:

- **Fixture tests.** `fixtures/synthetic-first-light.ndjson` and its truncated companion replay
  and project through the same code the viewer uses, and every anomaly they exist to demonstrate
  is asserted present. These are real tests.
- **Source-level guards.** Assertions that `viewer.js`, `viewer.html`, and `viewer.css` do not
  contain any HTML-parsing sink, any storage API, any remote URL, or any inline handler, and that
  they do contain the required absence phrasings, focus styling, and reduced-motion handling.

**The guards are not UI testing and are not described as such.** They read source text. They
cannot tell you that the page renders, that a mark lands where it belongs, or that focus survives
a keypress.

Headless-browser infrastructure was considered and declined. A driver, a browser binary in CI,
and the flake-management that comes with them would be several times the size of the thing under
test, for a page served to one browser on one machine. The trade is that visual and interaction
correctness is verified by hand, against the checklist below, and that the checklist has to
actually be run.

Two defects were found by running it during task:10 and are fixed: `style-src 'self'` silently
refused the inline `style` attribute positioning every map mark, stacking all of them at zero;
and rebuilding the ledger on selection destroyed the focused row, so arrow-key navigation
stopped after one step. Neither was visible to any guard, which is the argument for the
checklist in one sentence.

## 7. Manual smoke checklist

Deterministic, and runnable in about three minutes against the committed fixtures.

```sh
cargo build
./target/debug/witnessglass view --recording fixtures/synthetic-first-light.ndjson
```

**Boundary**

1. The URL printed is `http://127.0.0.1:<port>/?c=<64 hex chars>`.
2. Opening `http://127.0.0.1:<port>/` without the query gives `404 not found` and nothing else.
3. Opening `/projection.json` without the capability gives the identical 404.

**HUD**

4. The sensitive-recording warning is visible without scrolling.
5. Recording reads `complete`, 34 records, schema v2.
6. Recorder time reports the clock moving backwards once, citing `#31`, and says append order is
   unaffected.
7. Event kinds lists all nine v2 kinds. Every count has receipts or an explicit
   "no … record observed" with its scope.
8. Evidence gaps shows `duration_ms` supplied on 1 of 12, parent identity **never** supplied on
   any of 3, and the two standing caveats about unexercised surfaces and filesystem effects.
9. Anomalies lists seven, each with receipts.

**Map**

10. Two lanes: `identity not supplied` and `agent-synthetic-child-0001`. The unattributed lane is
    labelled as such, never as root.
11. Marks are spread across the track, not stacked at the left edge.
12. Anomalous marks carry a ring.
13. Switching to "Recorder time" changes mark positions, shows the DERIVED note, and leaves the
    ledger in canonical order 1…34.

**Ledger**

14. Rows run 1…34 in order.
15. Filtering to channel `reported` leaves exactly `#3`, `#22`, `#30`, and the count line says the
    canonical order is unchanged.
16. The payload-search checkbox is off, and its note says what turning it on reads.
17. Click a row; tab to it and press Enter; both select. Arrow Down/Up moves through consecutive
    rows and selection follows.

**Inspector**

18. Select `#3`. Reported and Observed appear as separately headed groups; the claim is quoted as
    a claim; `#2` is labelled "request — not proof of execution".
19. Select `#17`. Shape reads "ambiguous — nothing was paired", and `#16`, `#17`, `#18` appear as
    three separate cards with neither outcome chosen.
20. Select `#20`. Parent reads "parent identity not supplied — none is inferred from containment,
    adjacency, or timing".
21. Select `#28`. Expand the raw record: the markup-shaped payload is visible **as text**. No
    image, no alert, no styling change.
22. Click any receipt: it selects that record and the map mark follows.

**Truncation**

```sh
./target/debug/witnessglass view --recording fixtures/synthetic-truncated.ndjson
```

23. Completeness reads "ends mid-record", in alarm styling, with the fragment offset and length.
24. `session_ended` reads "no session_ended record observed" with the scope "the valid prefix of
    a recording that stops mid-record" — **not** the complete-recording scope.

**Accessibility**

25. Tab from the top: the skip link appears first and works.
26. Every focused control has a visible outline.
27. With reduced motion enabled at the OS level, nothing animates.
28. The page is usable in both light and dark system themes.
