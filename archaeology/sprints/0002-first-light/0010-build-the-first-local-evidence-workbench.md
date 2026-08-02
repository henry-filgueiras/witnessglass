---
id: tsk_01KZ2CEZ9GS0ZYR32VANEA75SD
sequence: 10
kind: task
status: pending
sprint: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
created: 2026-08-02
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
