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

**Three perspectives.** The first-use pass in task:11 found a single column of twelve peer
panels put the investigative loop too late in the hierarchy, so the workbench now leads with the
loop and keeps the rest one keystroke away. The perspective controls are real tabs: click, or
arrow keys, `Home`, and `End`.

### Events — the default

The whole investigative loop, above the fold on a 1440×900 window.

- **A compact summary**: completeness, record count, each tool-lifecycle kind counted
  individually and honestly, and the anomaly count as a button into Coverage. Completeness lives
  here, not only in Provenance, so relocating the detailed recording panel cannot hide damage.
- **The event map**: one mark per record, in lanes. Point events only. Append sequence by
  default; an optional recorder-time axis labelled derived, which never reorders the ledger.
  Channel is carried by shape — circle, diamond, square — as well as by colour and by the
  labelled legend. Anomalous marks are ringed; the selected mark grows. One tab stop for the
  whole map, with arrow keys moving between marks.
- **Search and compact filtering**: search always visible; a Filters disclosure carrying a count
  of active filters; removable chips for each active filter and a "clear all"; a visible
  anomaly-only shortcut; and the payload-search opt-in with its sensitivity warning at the point
  of use.
- **The canonical ledger**: append order, always. Sequence, channel, kind, tool, agent
  attribution, anomaly state. Correlation ids and secondary fields live in the inspector rather
  than consuming permanent width.
- **The evidence inspector**: a sticky companion to the ledger on wide screens, stacked beneath
  it below 1180px. Envelope, provenance, context, supplied duration and interruption where they
  exist, lifecycle subject where it exists, anomalies citing the record, correlated evidence as
  separate cards, and the raw payload collapsed until revealed.

Selection, search, filters, map axis, and inspector state survive perspective switches. They
live in memory only and die with the tab.

### Coverage — what was and was not captured

Evidence gaps and supplied-field coverage; every anomaly with receipts; per-kind counts across
the whole schema vocabulary including honest zero rows with their examined scope; subagent
boundary pairing; and the standing warnings that an unexercised surface is not a working
surface, that tool events do not reveal every file a session changed, and that two silences
agreeing is not corroboration.

### Provenance — where the recording came from

Recording identity and schema, recorder-time extrema and their caveat, channel counts, adapters
and mechanisms, and records by supplied agent identity.

### Everywhere

Every derived claim carries the raw sequence numbers supporting it, as clickable receipts, and
every zero carries the scope it was counted in. A receipt list longer than eight collapses
behind a disclosure naming how many records support the claim and builds its buttons when
opened — **collapsed is not deleted**, and the count is always visible.

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
- **An unexercised surface is not a working surface** and is not rendered as one. The denial
  rendering is the standing example: it has only ever met synthetic `tool_denied` records,
  because `PermissionDenied` has never fired against this project. Pass 3 established that an
  interactive refusal fires no hook at all and leaves a request with **no terminal record**, so
  what a reader sees for a denied call is a `tool_requested` that stops — indistinguishable in
  the interface, as in the recording, from an interruption or a crash. See
  [docs/claude-adapter.md](claude-adapter.md) §3.3.
- **A truncated recording's absences are scoped to its valid prefix**, not to a complete
  recording, everywhere the interface reports one.
- **No recording-semantic aggregate is computed in the browser.** JavaScript chooses no
  membership, no grouping, and no rollup: which records belong to a claim, which claims belong
  together, and what a count of them means are all decided in Rust and arrive with receipts and
  an examined scope. The summary renders each lifecycle kind individually and never sums them
  into an invented "outcomes" total, because a derived number without receipts belongs in Rust
  or nowhere.

  It does take the **cardinality of a receipt set Rust supplied** — "3 records" beside a claim
  whose receipts Rust listed — and it does compute **transient interface counts**, such as how
  many rows a filter or a search left showing. Neither invents a membership: the first counts a
  set it was handed, and the second describes the viewport rather than the recording. The line
  this draws is between counting what Rust decided and deciding what to count.

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

Deterministic, and runnable in a few minutes against the committed fixtures. Run it at roughly
1440×900 and again near 1024 wide, in both light and dark system themes.

```sh
cargo build
./target/debug/witnessglass view --recording fixtures/synthetic-first-light.ndjson
```

**Boundary**

1. The URL printed is `http://127.0.0.1:<port>/?c=<64 hex chars>`.
2. Opening `http://127.0.0.1:<port>/` without the query gives `404 not found` and nothing else.
3. Opening `/projection.json` without the capability gives the identical 404.

**Orientation — Events is the default**

4. The page opens on Events, with the tab marked selected.
5. The sensitive strip reads "Not redacted." and "Rendering is not redacting." without opening
   anything; expanding it gives the full explanation.
6. Without scrolling: the compact summary, the event map, the search box, the Filters control,
   the first ledger rows, and the inspector's relationship to the ledger are all understandable.
7. Summary reads `complete`, 34 records, and each lifecycle kind separately — 13 requested,
   10 succeeded, 2 failed, 1 denied — plus 7 anomalies. No "outcomes" total appears anywhere.
8. Clicking the anomalies figure moves to Coverage.

**Perspectives**

9. `Tab` to the tablist, then `ArrowRight`/`ArrowLeft`/`Home`/`End` move between perspectives and
   the selected one is visually and programmatically marked.
10. Set a search term, turn on Anomalous only, select a row; switch to Coverage and back. The
    search text, the filter chip, the row set, and the selection are all still there.

**Map**

11. Two lanes: `identity not supplied` and `agent-synthetic-child-0001`, each with a count. The
    unattributed lane is labelled as such, never as root.
12. Marks are small, spread across the track, and do not read as overlapping chips. Reported is a
    diamond, observed a circle, recorder a square — check with colour ignored.
13. Anomalous marks are ringed. The selected mark is visibly larger.
14. `Tab` reaches the map once, not once per record; `ArrowRight`/`ArrowLeft`/`Home`/`End` move
    between marks and the selection follows.
15. Switching to "Recorder time" changes mark positions, shows the DERIVED note, and leaves the
    ledger in canonical order 1…34.

**Ledger and filters**

16. Rows run 1…34 in order. Columns are sequence, channel, kind, tool, agent, anomaly.
17. The Filters control shows no badge when nothing is filtered. Open it, tick channel
    `reported`: the badge reads 1, a removable chip appears, exactly `#3`, `#22`, `#30` remain,
    and the count line still says canonical order is unchanged.
18. Non-matching map marks fade.
19. The chip's × removes that filter; "clear all" removes everything.
20. The payload-search checkbox is off and its note says what turning it on reads; turning it on
    changes the note to the ON warning.
21. Click a row; tab to it and press Enter; both select. `ArrowDown`/`ArrowUp` move through
    consecutive rows and selection follows.

**Inspector**

22. On a wide window the inspector sits beside the ledger and stays put while the ledger scrolls.
    Below about 1180px it moves underneath and neither is crushed.
23. Select `#3`. Reported and Observed appear as separately headed groups; the claim is quoted as
    a claim; `#2` is labelled "request — not proof of execution".
24. Select `#17`. Shape reads "ambiguous — nothing was paired", and `#16`, `#17`, `#18` appear as
    three separate cards with neither outcome chosen.
25. Select `#20`. Parent reads "parent identity not supplied — none is inferred from containment,
    adjacency, or timing".
26. Select `#28`. Expand the raw record: the markup-shaped payload is visible **as text**. No
    image, no alert, no styling change.
27. Click any receipt: it selects that record and the map mark follows.

**Coverage and Provenance**

28. Coverage shows `duration_ms` supplied on 1 of 12, parent identity **never** supplied on any
    of 3, the seven anomalies with receipts, the whole event-kind vocabulary, and both standing
    caveats.
29. A long receipt list is collapsed as "N supporting records"; expanding it reveals N clickable
    receipts, and clicking one returns to Events with that record selected.
30. Provenance shows session, schema, completeness, recorder-time extrema with their caveat,
    channels, capture points, and records by supplied agent identity.

**Truncation**

```sh
./target/debug/witnessglass view --recording fixtures/synthetic-truncated.ndjson
```

31. The Events summary reads "ends mid-record" in alarm styling, with the fragment offset and
    length, and says every absence is scoped to the valid prefix. Damage is visible without
    opening Provenance.
32. In Coverage, `session_ended` reads "no session_ended record observed" with the scope "the
    valid prefix of a recording that stops mid-record" — **not** the complete-recording scope.

**Accessibility**

33. Tab from the top: the skip link appears first and works.
34. Every focused control has a visible outline.
35. With reduced motion enabled at the OS level, nothing animates.
36. The page is usable in both light and dark system themes, and every distinction that matters
    survives with colour ignored.
