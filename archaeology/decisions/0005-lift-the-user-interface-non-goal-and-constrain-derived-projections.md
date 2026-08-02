---
id: dec_01KZ2BMSPYK5YNKSGP1Z5TS4M6
sequence: 5
kind: decision
status: accepted
created: 2026-08-02
---

# Lift the user-interface non-goal and constrain derived projections

## Context

`CLAUDE.md` §6 listed "a TUI or web UI or dashboard" among the bootstrap non-goals, alongside
a daemon, an MCP server, a distributed collector, a plugin framework, an AI summarizer, PID
attachment, crate publication, and multi-agent coordination. The list existed to stop the
project designing a presentation layer ahead of a working kernel, which is the standard way an
evidentiary tool ends up with a schema shaped by what was easy to render rather than by what
was true.

Two things made that the right call at bootstrap. There was no recording kernel, so a view
would have been a view of nothing. And there was no real recording, so any view would have
been built against an imagined session — which is exactly how a tool acquires a rendering that
looks correct on fixtures and misrepresents reality.

Both conditions have now changed, and changed by measurement rather than by opinion.

sprint:1 closed with all seven success criteria met. The kernel exists, replays
deterministically, and has explicit tested behaviour for versioning, damage, ordering, and
truncated tails. task:4 recorded one real Claude Code session end to end — 234 records, schema
v2, structurally complete — and compared what the integration promised against what it
delivered. There is now a concrete artifact to project *from*, and, more importantly, a
measured list of what that artifact does and does not contain.

The stronger argument for lifting the non-goal is not that a UI is now allowed. It is that
first contact produced findings that are **hard to see in raw NDJSON and easy to see in a
projection**, and some of them are findings about the recording's own limits:

- 82 tool requests pairing with 82 completions, with no unmatched record in either direction.
- A subagent's 27 tool calls, attributable to it by `context.agent_id`, nested inside the
  parent's single `Agent` call — an attribution that exists in the data and is invisible when
  reading lines.
- A `subagent_stopped` with no matching `subagent_started`, which is precisely the kind of
  asymmetry a rendering surfaces immediately and a reader scrolling a file does not.
- An agent's reported intent sitting next to the observed command it describes, which is the
  project's entire thesis and currently requires `jq` to see at all.

The argument *against* lifting it has not gone away, and it is recorded here rather than
resolved, because the constraints below exist to hold it:

**A view is the surface where partial coverage gets mistaken for complete observation.**
dragon:1's resolution criteria say a recording must carry fidelity provenance "such that no
consumer can mistake partial coverage for complete observation". Raw NDJSON is hostile enough
that nobody mistakes it for a complete account of a session. A clean timeline is not. The
recorded session changed a tracked file with a shell redirect and reformatted another with a
formatter, and **neither produced a mutation event**; a view that renders "files changed" from
tool events would be confidently wrong, and would look authoritative while being so.

**The one recording available is of a session where nothing went wrong.** No tool failure, no
permission denial, no interruption, no resume. A projection built and validated only against
it will render the happy path well and has no evidence at all about the rest.

**A view renders unredacted material.** dragon:2 stands: nothing is redacted, and rendering is
not redacting. A projection makes a sensitive recording *easier* to read, which is the point,
and also easier to screenshot, share, and paste, which is not.

## Decision

**Lift exactly one bullet from `CLAUDE.md` §6: "a TUI or web UI or dashboard".** Derived
projections over a recording — spans, timelines, landmarks, correlated views, and a local
presentation layer over them — are in scope from now on.

**Nothing else on that list is lifted.** A daemon, background service, or distributed
collector; an MCP server; a generalized plugin or adapter framework before two real adapters
exist; an AI summarizer of recordings; arbitrary PID attachment or OS-wide tracing; crate
publication; and multi-agent coordination all remain non-goals, and each would need its own
decision. In particular, a presentation layer must not smuggle in a daemon: a local, on-demand
renderer over a recording on disk is what is permitted, and a background process that watches,
collects, or serves is not.

Every projection built under this decision carries five conditions. They are not new policy;
each is an existing constraint restated at the surface where it will actually be tested.

**1. Derived and disposable.** `CLAUDE.md` §3 already governs this. A projection must be fully
rebuildable from the raw stream, must never rewrite, overwrite, or "clean up" raw evidence,
and must be safe to delete entirely without losing anything. If a projection holds a fact the
raw stream cannot regenerate, either the raw stream is missing something or the projection is
inventing something, and both are defects.

**2. The channel distinction must survive into the rendering.** decision:2 and decision:3 make
reported and observed unrepresentable as each other in the *data*. A view can undo that with
styling alone: rendering an agent's `reported_intent` and an observed `tool_succeeded` with
equal visual weight, or as one merged "step", promotes a claim into a fact by presentation. A
reader of a projection must be able to tell, without effort, which channel any element came
from — and a projection that renders a disagreement between the channels must show both, not
pick one.

**3. Absences must be rendered as absences.** This is the condition most likely to be violated
by accident, and it is the one dragon:1 cares about. A projection must not fill a gap with a
plausible value, must not infer parentage from adjacency or containment when
`parent_agent_id` was not delivered, must not present tool-derived file changes as a complete
account of what the session changed, and must not let a surface that was never exercised
render as one that works. "We did not see this" has to be visible in the view, not only in the
documentation.

**4. No segmentation by `prompt_id`.** dragon:3 is open. Until it is settled, no projection may
group work by `prompt_id` and no view may describe a recording as containing N turns.
`tool_use_id` (82 correlated pairs, the one identifier whose semantics this project has
actually tested) and `agent_id` are the identifiers currently licensed by evidence.

**5. Local only, and no claim of shareability.** A projection runs against a recording on the
machine that produced it. It gains no export format, no "share" affordance, no hosted mode,
and no wording that implies the rendered output is safer than the recording behind it. It is
not. dragon:2 stands unchanged, and this decision does not weaken it by a word.

## Consequences

- **The next sprint may build a projection layer.** What form it takes — a library, a CLI
  renderer, a TUI, or a locally served page — is deliberately not decided here. That is a
  design question for the sprint that does it, and pre-deciding it from an archaeology
  artifact would be the same mistake the non-goal list existed to prevent.
- **Projections become a place the epistemic invariant can be violated,** and the violation
  will be easier and less visible than in the schema, because it will look like a styling
  choice. The five conditions above are the testable form of that risk. A projection that
  merges channels, invents a hierarchy, or hides an absence is a defect of the same class as a
  schema that permits it, and should be treated with the same seriousness.
- **A projection built now is validated against exactly one clean session.** That is a real
  limitation on any confidence it earns, and it argues for recording a deliberately hostile
  session — one that fails, is denied, is interrupted, resumes, and spans several prompts —
  either before or alongside the projection work. Such a recording would also answer dragon:3's
  first-order question outright.
- **`CLAUDE.md` §6 is now a list with one item removed from it,** and its authority depends on
  lifts being deliberate rather than incremental. This is the first. The remaining items stand
  at full strength, and none is weakened by precedent: a lift is evidence that the list works,
  not evidence that it is soft.
- **The README's "what does not exist" line stays accurate** and stays as it is. Nothing has
  been built; permission to build is not a feature.

### Deliberately deferred

- **The form and scope of the first projection.** Left to the sprint that builds it.
- **Whether a projection may be persisted to disk at all,** or must be recomputed on demand.
  Both satisfy condition 1; the trade-off is not worth pre-empting without an implementation
  in front of it.
- **Any export, sharing, or screenshot-safe rendering path.** Blocked by dragon:2, not merely
  postponed. It cannot be unblocked by a projection; it needs a capture and redaction contract
  that does not exist.
- **Whether a projection may be served over a local HTTP port.** A locally served page is a
  plausible form and would sit uncomfortably close to the daemon non-goal, which is not lifted.
  Decide it explicitly if the sprint wants it, rather than arriving there by convenience.
- **Rendering of recordings large enough to matter.** Replay reads the whole file into memory
  (decision:3), and nothing here changes that.
