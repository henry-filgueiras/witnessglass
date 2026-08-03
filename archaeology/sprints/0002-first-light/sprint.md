---
id: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
sequence: 2
kind: sprint
status: closed
created: 2026-08-02
closed: 2026-08-03
---

# First light

## Goal

Make one closed WitnessGlass recording meaningfully inspectable through a private local
browser, without moving the recording off the machine, mutating raw evidence, or hiding the
distinction between reported, observed, and derived claims.

## Rationale

First contact left a structurally complete recording of 234 records and 580 KB that is, in raw
NDJSON, close to unreadable. Every finding in task:4 that mattered — 82 requests pairing with
82 completions, a subagent's 27 tool calls attributable by `context.agent_id`, a
`subagent_stopped` with no matching start, an agent's reported intent sitting beside the
observed command it describes — was extracted with `jq` by a session that had to characterize
the file before it could begin reading it. decision:5 lifted the interface non-goal on exactly
that evidence. This sprint spends the lift on one thing.

The intended architecture, and the direction of authority within it:

```text
canonical NDJSON
      ↓
Rust replay and validation
      ↓
receipt-bearing inspection projection
      ↓
capability-protected loopback HTTP
      ↓
bundled browser workbench
```

Rust owns validation, schema interpretation, damage handling, correlation, and projection. The
browser is a renderer and an interaction surface. It must not parse NDJSON independently,
redefine lifecycle semantics, or invent a correlation the projection did not license. A second
implementation of what a recording says is a second opinion about what a recording says, and
this project has room for exactly one — the reasoning task:7 used to keep a parser out of
`check-recording.sh`, arriving one layer up.

The command being worked toward:

```sh
witnessglass view --recording <PATH>
```

Foreground and short-lived, on an OS-selected loopback port, normally opening a browser, with a
`--no-open` path for tests and remote terminals. It dies with the process that started it.
**That is explicitly not a daemon,** and decision:5's warning that a presentation layer must not
smuggle one in names the constraint this sprint is most likely to violate by convenience rather
than by intent.

decision:5 deliberately deferred whether a projection may be served over a local HTTP port,
observing that it "would sit uncomfortably close to the daemon non-goal" and asking that it be
decided explicitly rather than arrived at. This sprint answers it: yes for a foreground process
serving one explicitly supplied snapshot to loopback behind a per-launch capability, no for
anything that outlives the invocation, watches a file, or listens anywhere else. The accepted
decision recording that boundary belongs with the projection task, written once the boundary is
concrete and tested — not as a restatement of this plan.

What the viewer may claim is fixed by task:4 rather than by taste. Append sequence is the only
canonical total order (decision:3). Recorder timestamps are descriptive and may be equal or move
backward. `tool_use_id` is the only correlation whose behaviour has been exercised convincingly,
across 82 pairs. Successful first contact supplied no `duration_ms` at all. A supplied `agent_id`
can attribute subagent work, but an absent one does not prove a root agent. `parent_agent_id`
never arrived, so parentage may not be recovered from containment or timing. A subagent stop
arrived with no observed start. Tool hooks reveal no descendant filesystem effect and do not
distinguish parallel dispatch from serialized dispatch. dragon:3 forbids segmenting by
`prompt_id` or describing a recording as containing N turns. Reported intent and observed tool
evidence may correlate and remain separate records on separate channels.

**The first viewer exposes those limits. It does not repair them cosmetically.** A clean
rendering is the surface where partial coverage is most easily mistaken for complete
observation, which is the whole of decision:5's argument against itself.

The recording is also highly sensitive and nothing is redacted (dragon:2). Rendering is not
redacting, and no wording anywhere in this slice may imply otherwise.

Scale is settled by the specimen: 234 records, 580 KB. The target is making that real recording
intelligible, not a hypothetical large one.

The visual direction may be opinionated — a precise optical instrument or a flight-recorder
workbench rather than a generic admin dashboard. That direction never outranks accessibility or
evidentiary honesty, and where it collides with either, it loses.

## Success criteria

**Projection**

- A pure Rust inspection projection, rebuildable entirely from replayed raw records, never
  rewriting raw evidence, and safe to delete.
- Every derived tool lifecycle, anomaly, aggregate, and grouping retains the raw sequence
  numbers supporting it. A derived claim that cannot produce its receipts is a defect.
- Existing v1 and v2 recordings both remain replayable and inspectable, without flattening their
  different vocabularies into a false common meaning.
- Complete and truncated recordings are both viewable, with truncation unmissable and the valid
  prefix preserved.
- A corrupt recording fails before a viewer presents partial content as trustworthy.

**Boundary**

- A loopback-only, read-only, per-launch server exposing exactly one explicitly supplied
  recording snapshot.
- Every endpoint carrying recording data is protected by an unguessable per-launch capability.
  Loopback binding alone is not treated as sufficient protection.
- No runtime network dependency: no CDN, telemetry, upload, external font, or third-party asset.
- Static browser assets are bundled with the binary.
- No browser persistence of recording data: no service worker, local storage, analytics, or
  intentional caching.
- Recording-controlled strings are treated as hostile input — rendered as text, never as
  executable HTML, Markdown, URL, CSS, or script.

**Interface**

- Four surfaces: a session/evidence HUD, an event-point map, a canonical event ledger, and an
  evidence inspector, plus filters and search.
- Direct navigation from every derived claim to its supporting records.
- Recorder-order view is the default.
- Any timestamp projection is clearly marked derived. It may position point events by recorder
  timestamp, but must not reorder the canonical ledger and must not portray request-to-outcome
  spacing as execution duration.
- Missing evidence is phrased honestly: "no failure record observed", not "no failure occurred";
  "outcome not observed", not "still running"; "agent identity not supplied", not "root agent";
  "stop without observed start", not an invented explanation.
- Reported, observed, and derived information are distinguishable by label and structure, not by
  colour alone.
- Full payloads are collapsed by default, so opening the viewer does not immediately wallpaper
  the screen with source, commands, and output.
- Keyboard navigation, visible focus, sufficient contrast, and reduced-motion behaviour.

**Evidence**

- A final private first-use pass against the original first-contact recording. That recording
  stays untracked, unquoted, unexcerpted, and local; only aggregate findings and usability
  observations enter this repository.
- Documentation stating exactly what the viewer shows, what it derives, and what it cannot know.

## Non-goals

- Hosted access, remote binding, collaboration, accounts, or uploads.
- Redaction, sanitization, export, download, or shareable HTML.
- Live capture, file watching, tailing, or automatic refresh.
- Editing, annotation, bookmarks, or any mutation of a recording.
- Cross-session comparison or indexing.
- AI-generated summaries or findings.
- A TUI.
- A database, daemon, background collector, or generalized frontend framework.
- Prompt or turn grouping, per dragon:3.
- Inferred root agents, parentage, causality, concurrency, execution duration, or filesystem
  effects.
- A flame graph or span hierarchy whose structure the evidence does not establish.
- Performance work for arbitrarily large recordings.
- Production-grade visual polish or a hosted-product architecture.

## Outcome

Four tasks, all closed. Two accepted decisions. 168 tests behind the same single gate CI runs.
One real recording made meaningfully inspectable through a private local browser, without the
recording moving, changing, or being quoted anywhere.

### Success criteria, against evidence

**Projection**

- **A pure Rust inspection projection, rebuildable from raw, never rewriting it, safe to delete.**
  task:8, decision:6. `inspect` is a pure function of a validated `Replay` that reads no file,
  consults no clock, and *borrows* the raw records rather than owning them — so "never rewrites
  raw evidence" is a property of the types rather than a discipline.
- **Every derived entity retains the raw sequence numbers supporting it.** Enforced by
  construction: there is no constructor that produces a derived claim without receipts. 35 tests,
  one of which walks every receipt in a deliberately anomalous recording and asserts each names a
  real record.
- **v1 and v2 both inspectable without flattening their vocabularies.** `CorrelationId` and
  `ToolEvidence` are schema-tagged enums, so a v1 `tool_call_id` and a v2 `tool_use_id` spelled
  identically are different keys, and v1's claim of a witnessed *beginning* never becomes v2's
  request.
- **Complete and truncated both viewable, truncation unmissable, valid prefix preserved.**
  `ExaminedScope` carries tail state, so every absence reads against the population it was found
  in. On the truncated fixture the Events summary leads with "ends mid-record" in alarm styling
  and `session_ended` reads "no record observed" scoped to the valid prefix.
- **A corrupt recording fails before a viewer presents partial content.** `Snapshot::load`
  replays before a listener is bound; the command exits non-zero at a terminal without printing a
  URL.

**Boundary**

- **Loopback-only, read-only, per-launch server over one supplied snapshot.** task:9. One
  `TcpListener::bind((Ipv4Addr::LOCALHOST, 0))` in the crate, with no flag, environment variable,
  or test hatch that binds elsewhere.
- **Every endpoint capability-protected; loopback not treated as sufficient.** 256 bits from
  `/dev/urandom` per launch, no weaker fallback, required on all four routes. An unauthorized
  request and an unknown path return byte-identical 404s.
- **No runtime network dependency, assets bundled, no browser persistence.** Three
  `include_str!` assets, zero remote references, zero storage APIs — asserted by guard and
  confirmed live in a browser with `localStorage`, `sessionStorage`, cookies, and the service
  worker controller all empty.
- **Recording-controlled strings treated as hostile.** `textContent` is the only path content
  takes; a payload containing `</script><img src=x onerror=alert(1)>` produces zero injected
  elements and renders as text.

**Interface**

- **The four surfaces exist**, reorganized in task:11 into three perspectives after the first-use
  pass: the event map, the canonical ledger, and the evidence inspector together in Events, with
  the HUD's content split between a compact summary in Events and the Coverage and Provenance
  perspectives. This is a change in arrangement, not in content — every panel the criterion named
  is present with its receipts. It is recorded here as a deliberate reading of "four surfaces"
  rather than a silent one.
- **Direct navigation from every derived claim to its supporting records.** Receipts are buttons
  everywhere, including inside collapsed sets.
- **Recorder-order view is the default**, and the derived recorder-time axis never reorders the
  ledger — asserted in a browser, not only in source.
- **Missing evidence phrased as the sprint required.** "no failure record observed", "outcome not
  observed", "agent identity not supplied", "stop without observed start" — each verified present
  on the real specimen, along with the absence of "still running", a turn count, a root agent, an
  execution duration, a files-changed list, and any claim that nothing failed.
- **Reported, observed, and derived distinguishable by label and structure.** A glyph, a word,
  and — on the map — a shape. Remove colour and every distinction survives.
- **Payloads collapsed by default.**
- **Keyboard navigation, visible focus, contrast, reduced motion.** Roving tabindex on both the
  tablist and the map; arrow keys through ledger rows and marks; every semantic colour measured
  at ≥5.1:1 in dark and ≥5.2:1 in light after task:11 raised `--faint` off 4.17.

**Evidence**

- **A final private first-use pass against the first-contact recording.** task:11. It reproduced
  every task:4 finding — 82 correlated pairs, 65 reported intents, the unmatched subagent stop,
  `duration_ms` on 0 of 82, parentage on 0 of 3, `prompt_id` on 233 of 234 — and the recording's
  SHA-256 was identical before and after. Nothing derived from it entered this repository.
- **Documentation stating what the viewer shows, derives, and cannot know.** `docs/viewer.md`,
  including a 36-point manual smoke checklist and an explicit account of what the guards are not.

### What first light actually settled

Two independent methods over one file agreed exactly. task:4 characterized the recording by hand
with `jq`, in a session that had to reconstruct the file's shape before it could read it; the
viewer produced the same numbers in 27 ms of startup and a few milliseconds per interaction. That
agreement is the strongest evidence available that the projection is honest, and it is worth more
than any property of the interface.

The sprint also demonstrated its own thesis against itself. decision:5 lifted the interface
non-goal while arguing that a clean rendering is exactly where partial coverage gets mistaken for
complete observation. The interface built under that lift now volunteers, unprompted, that a
surface this recording did not exercise is not a working surface, that tool events do not reveal
every file a session changed, and that two silences agreeing is not corroboration.

### What the sprint deliberately did not build

Every non-goal held. No hosted access, remote binding, collaboration, accounts, or uploads. No
redaction, sanitization, export, download, or shareable HTML — and no wording anywhere implying a
rendered view is safer than the recording behind it. No live capture, file watching, tailing, or
refresh. No editing, annotation, or bookmarks. No cross-session comparison or indexing. No AI
summaries. No TUI. No database, daemon, background collector, or frontend framework — and no
package manager, bundler, or new runtime dependency of any kind; the crate still carries the same
three it had at the start of the sprint. No prompt or turn grouping. No inferred root agents,
parentage, causality, concurrency, execution duration, or filesystem effects. No flame graph or
span hierarchy. No performance work for recordings nobody has — the numbers were taken and
deliberately not acted on.

### Dragons

**All three stay open, and none was weakened.**

dragon:1's coverage question is untouched by this sprint: the viewer reports what the adapter
captured and is loud about what it did not. dragon:2 is, if anything, sharper — a recording is now
much easier to read, which was the point, and also easier to screenshot, which is why the
sensitive strip is persistent and there is no export path. dragon:3 constrained the whole design:
`prompt_id` survives on its record with "It groups nothing: what it delimits is unestablished",
and defines no group, filter, lane, or segment anywhere.

### Scarp desire paths

idea:1 recurred four more times, eleven for eleven across both sprints, and the sample is now
closed — the friction is real, the count is no longer informative. idea:2 and idea:3 did not
recur in this sprint. No new idea was filed and nothing was sent upstream.

### What the next sprint inherits

A working viewer with one honest gap in its evidence, and it is the same gap first contact left.
**Everything the projection and the interface do for failure, denial, interruption, resume, and
multi-turn is exercised by synthetic fixtures only.** The one real recording available is a clean
session where nothing went wrong, and it has now been used twice.

A deliberately hostile recording — one that fails, is denied, is interrupted, resumes, and spans
several distinct prompts — is the highest-value next measurement available. It would test the
paths this sprint built on faith, and it would answer dragon:3's first-order question outright
with no schema change and no code. It was proposed at the start of this sprint and deliberately
deferred so that the happy path could be seen working first; it should not be deferred twice.
