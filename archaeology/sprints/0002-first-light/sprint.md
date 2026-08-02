---
id: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
sequence: 2
kind: sprint
status: active
created: 2026-08-02
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
