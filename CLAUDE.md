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

- Raw append-only session recordings are the canonical product data. Append-oriented,
  immutable once written, likely JSONL/NDJSON.
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

## 6. Bootstrap non-goals

Do not build, and do not lay speculative groundwork for:

- a daemon or background service
- an MCP server
- a TUI or web UI or dashboard
- a distributed collector
- a generalized plugin/adapter framework (no framework before two real adapters exist)
- an AI summarizer of recordings
- arbitrary PID attachment or OS-wide tracing
- crate publication
- multi-agent coordination

Foundation-level direction in the README is context, not a license to design a speculative
framework ahead of a working kernel.

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
