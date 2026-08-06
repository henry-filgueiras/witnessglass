# CLAUDE.md — agent contract for WitnessGlass

This file is the durable working contract for any agent (or human) doing work in this
repository. Read it before substantial work. It outranks convenience.

## 1. Project and name

- The project in prose is **WitnessGlass**.
- The repository is `henry-filgueiras/witnessglass`.
- The crate, library, and eventual binary are all `witnessglass`.

Use those exact forms. Do not introduce alternative spellings, casings, or short names.

The pre-bootstrap working name *AgentScope* is **not** an alias for this project and must
not be used as one. It may appear only in historical context that explains the rename and
links the unrelated established framework at <https://github.com/agentscope-ai/agentscope>.

Positioning line, used verbatim where a one-liner is needed:

> WitnessGlass is a flight recorder for coding agents: declared intent, observed activity,
> and temporal replay.

The name refers to an optical-manufacturing witness glass: a sample exposed alongside a
process and retained for later measurement and traceability.

## 2. Core epistemic invariant

This is the load-bearing constraint of the whole project. If a change would violate it, the
change is wrong, however convenient it looks.

- **Reported** information (what the agent says: intent, hypotheses, decisions, friendly
  descriptions of work) and **observed** information (what the machinery sees: tool
  lifecycle, commands, exit status, file mutation, test execution) are distinct epistemic
  channels.
- Neither channel is silently promoted into the other. Cooperative intent is not ground
  truth. Process facts do not reconstruct intent.
- Every event must retain source and fidelity provenance, so a later reader can always tell
  which channel a claim came from and how much that channel could actually see.
- Correlation may derive a view. It may never rewrite, overwrite, or "clean up" raw
  evidence. A disagreement between channels is a finding to preserve, not a bug to
  reconcile away.

## 3. Canonical and derived state

- Raw append-only session recordings are the canonical product data: UTF-8 NDJSON, one
  complete record per newline-terminated line, one file per session, immutable once
  written. Settled by decision:3 and refined by decision:4.
- Spans, timelines, landmarks, findings, summaries, and any dashboard are **derived
  projections**. They must be rebuildable from the raw stream and must be treated as
  disposable.
- If a projection cannot be rebuilt from raw events, either the raw stream is missing
  something or the projection is inventing something. Both are defects.

## 4. Capture boundary

- Cooperative hooks or an explicit semantic emission API are the primary sensor.
- OS/process observation is optional, secondary, and corroborating. It is not the v0
  mechanism and is not a substitute for cooperation.
- v0 must not claim it can attach to an arbitrary agent process and observe every
  descendant process.
- Never claim complete observation without evidence. Each adapter documents its fidelity
  and its blind spots explicitly. "We did not see this" is a supported, publishable result;
  an unsupported completeness claim is not.

## 5. Privacy posture

- Session data may contain source code, prompts, commands, absolute paths, command output,
  and secrets. Treat every recording as potentially sensitive.
- Do not commit real recordings to this repository. Fixtures must be synthetic and
  obviously so.
- Do not describe recordings as redacted, sanitized, or safe to share until a concrete
  capture and redaction contract exists, is implemented, and is tested. No magical or
  best-effort redaction claims.

## 6. Standing non-goals, and the one item lifted from them

Do not build, and do not lay speculative groundwork for:

- a daemon, background service, or anything that outlives the command that started it
- an MCP server
- a distributed collector
- a generalized plugin/adapter framework (no framework before two real adapters exist)
- an AI summarizer of recordings
- arbitrary PID attachment or OS-wide tracing
- crate publication
- multi-agent coordination
- hosted access, remote binding, upload, or export of a recording in any form
- a redaction or safe-sharing implementation, or any claim that a recording — or a rendering
  of one — is safe to share
- a speculative causal hierarchy: inferred parentage, root agents, concurrency, execution
  duration, or filesystem effects the evidence does not establish

Foundation-level direction in the README is context, not a license to design a speculative
framework ahead of a working kernel.

**One item has been lifted from this list: a TUI, web UI, or dashboard.** decision:5 lifts it,
on the evidence that a working kernel and one real recording now exist to project from. The
remaining items stand at full strength and none is weakened by that precedent; each would need
its own decision.

**What that lift currently authorizes is one thing.** sprint:2 spends it on a foreground,
loopback-only, read-only local viewer — `witnessglass view --recording <PATH>` — which
validates and projects one explicitly supplied recording, serves that immutable snapshot to a
browser on an OS-selected loopback port behind an unguessable per-launch capability, and exits
with the process that started it. Any other form decision:5 would also permit — a TUI in
particular — is not authorized work and needs its own sprint. A lift is not a standing budget.

**The browser is downstream of Rust.** Replay, validation, schema interpretation, damage
handling, correlation, and projection belong to Rust. A rendering layer renders: it does not
parse raw NDJSON, redefine lifecycle semantics, or invent a correlation the projection did not
license. Two implementations of what a recording says are two opinions about what a recording
says, and this project has room for exactly one.

A projection built under decision:5 carries five conditions, and they are load-bearing:

- **Derived and disposable** — rebuildable from raw, never rewriting raw, safe to delete (§3).
- **The channel distinction survives into the rendering.** Reported and observed must stay
  visibly distinct in the view, and a disagreement between them is shown, not resolved. A
  merged "step" promotes a claim into a fact by presentation alone (§2).
- **Absences are rendered as absences.** No gap filled with a plausible value; no parentage
  inferred from adjacency or containment; no unexercised surface rendered as a working one;
  no tool-derived list of changed files presented as a complete account of what changed. This
  is the condition most likely to be broken by accident.
- **No segmentation by `prompt_id`,** and no describing a recording as containing N turns,
  while dragon:3 is open. `tool_use_id` and `agent_id` are the identifiers evidence licenses.
- **Local only.** No export, no share affordance, no hosted mode, and no wording implying
  rendered output is safer than the recording behind it. Rendering is not redacting (§5).

A presentation layer must not smuggle in a daemon. A local, on-demand renderer over a
recording on disk is permitted; a background process that watches, collects, or serves beyond
its own invocation is not.

## 7. Scarp workflow

Project archaeology lives in `archaeology/` and is managed with Scarp (currently `0.2.0`).

- Before substantial work, read this file, the active sprint, pending tasks, relevant
  decisions, and open dragons.
- Use Scarp commands (`scarp new`, `scarp close`, `scarp list`, `scarp show`) for creation
  and lifecycle transitions. Do not hand-allocate sequence numbers, slugs, stable ids,
  paths, or front matter — Scarp owns those.
- Prefer `scarp new ... --body-file <path>` with a temporary UTF-8 Markdown file whose
  `## ` headings match the collection's own sections.
- Run `scarp doctor` before completing work. `scripts/check.sh` already includes it.
- **Preregistering an experiment: decision:7 governs how criteria are written.** Every prediction,
  feasibility check, and verdict rule names the exact quantity the code will compute, and the
  preregistration carries an explicit propagation pass over every mechanism the feasibility check
  found. Eight rounds were spoiled by a criterion that did not mean what it said.
- **Using a real recording as evidence: decision:8 governs which are admitted and what may be
  reported.** Mechanically derived counts and frequencies may be published; contents may not, including
  to make evidence easier to inspect.
- Preserve history. Append results and follow-ups; do not rewrite previous conclusions to
  make them look correct in hindsight.
- Product recordings are runtime data. Scarp archaeology is durable project knowledge.
  Never dump session transcripts, logs, or recordings into `archaeology/`.

## 8. Desire-path dogfooding

WitnessGlass is an external case study for Scarp, and the evidence is only worth anything
if it is real.

- Record only friction actually encountered while doing genuine WitnessGlass work. Do not
  invent, dramatize, or go looking for friction.
- In the active task's `## Result`, capture: the workflow attempted, the friction observed,
  the workaround used, and the smallest useful affordance that would have removed it.
- If a piece of friction recurs, or is independently useful beyond this project, promote it
  to a local Scarp idea titled `Scarp: …`.
- Do not interrupt WitnessGlass work to modify Scarp. Scarp itself is out of bounds here.
- Do not send anything upstream to Scarp without explicit human authorization.

## 9. Checks and completion

- `scripts/check.sh` is the local gate. CI invokes the same script so the two cannot drift.
- Commit completed vertical slices once the gate passes.
- Do not commit incomplete work merely to mark progress, and do not weaken a check to make
  it pass.

## 10. Git authority

- Commits are expected for completed work.
- **Pushes require task-specific human authorization.** A previous authorization does not
  carry forward.
- The inaugural bootstrap prompt authorized only the bootstrap pushes. That authorization
  is spent and does not extend to later work.
- Publishing a crate, creating a tag, and cutting a release always require separate
  explicit authorization, every time.
- Do not add branch protection, secrets, deployments, or external services on your own
  initiative.
