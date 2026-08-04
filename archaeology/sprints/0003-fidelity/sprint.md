---
id: spr_01KZ50ZMMD79D5FS65F4CTV9C2
sequence: 3
kind: sprint
status: closed
created: 2026-08-03
closed: 2026-08-04
---

# Fidelity

## Goal

Make this project's claims about what the Claude integration sends verifiable rather than
assumed, so that "we did not see this" can be distinguished from "we did not look correctly".

## Rationale

dragon:1 carried a finding for two sprints that was wrong. The hook-level duration was reported
as never supplied across 82 completions, confirmed by a follow-up whose three checks all rested
on the adapter being right about the key's name, and repeated in the README and in
`docs/claude-adapter.md`. The integration had been sending `duration_ms` on every completion from
the beginning. The adapter read `duration`, ignored unknown fields by design, and the confirming
scan ran over the recording — which is the adapter's own output, downstream of the field selection
under test.

The error was not carelessness. Every individual step was reasonable and the conclusion was still
false, because the whole chain was inside one system asking itself whether it was complete. What
broke it was an observation taken from outside: a raw-payload probe that shares no code with the
adapter, which answered the question in minutes on its first run.

That generalises past `duration_ms`. Every "the integration never sent X" statement this project
holds was established the same way, and `parent_agent_id` — the basis for the standing refusal to
infer parentage — is the largest of them. Those conclusions are not now believed to be false. They
are unconfirmed, which is a different and more honest status, and the difference is worth closing.

This sprint was commissioned mid-round, after the work that provoked it had already been
committed. That work is context rather than sprint scope:

- `1b655ac` — adapter reads `duration_ms` and `is_interrupt`; probe stops asking about one spelling
- `cc2d58b` — README and adapter reference corrected
- `8bbc96c` — dragon:1, dragon:2, and dragon:3 record what the hostile session found
- `6d0d4d1` — `docs/hostile-recording-pass-3.md`, the executable third-pass runbook

## Success criteria

- The adapter can state, at compile time, what it records and what it drops on purpose, with a
  reason for each dropped field. A field in neither category is detectable rather than silent.
- Detecting one is opt-in and non-default, because a recorder that stops when its upstream adds a
  field is worse than one that misses the field.
- The mechanism is documented beside the leniency it qualifies, including where it is weaker than
  observing the wire directly.
- Nothing in this sprint changes what a recording claims, what the schema holds, or what the
  projection derives. Correctness of the capture boundary only.

## Non-goals

- Making strict validation the default, or arming it by default. It is a canary.
- Modelling `permission_mode`, or any other deliberately unrecorded field. dragon:1 argues for
  `permission_mode` specifically and that argument needs a decision, not a quiet field addition.
- Resolving dragon:1, dragon:2, or dragon:3. This sprint improves the instruments; the
  measurements those dragons want still need a session that has not been run.
- Any change to the projection, the viewer, or the record schema.

## Outcome

Two tasks, both closed. No decision, no schema change, and — verified rather than asserted — no
non-comment line changed under `src/`. The sprint improved the instruments and the statements
made with them; it deliberately did not take the measurements those instruments are for.

### Success criteria, against evidence

- **The adapter can state, at compile time, what it records and what it drops on purpose, with a
  reason for each dropped field.** task:12. `HookPayload` grew a flattened `unmodelled` map, so
  the set of modelled fields *is* the struct and no second list can drift from it, and
  `DELIBERATELY_UNRECORDED` names what is dropped, each entry with its reason. The pair is a
  complete, reviewable statement of everything this adapter has seen on the wire.

  **The list was wrong when it was written, and the sprint caught it.** Two of its seven entries
  — `model` and `stop_reason` — came from the hooks reference rather than from a payload, in the
  same commit as a mechanism whose purpose is to stop the adapter believing the hooks reference.
  Pass 3 captured `SubagentStop` for the first time, `stop_reason` was not there, and the rule
  now stated in the code is that a field is listed only after being observed. The addendum in
  task:12 records which entries changed and why. The rule is a convention enforced by review,
  not by the compiler, and `model` and `reason` sit unaccounted-for on purpose as its live test.

- **Detecting one is opt-in and non-default.** `--strict-json-validation` and
  `WITNESSGLASS_STRICT_JSON=1`; the default path is unchanged and a test says so. It has fired
  in earnest exactly once: pass 3's two `SubagentStop` payloads, refused for four fields nobody
  had looked at — the behaviour it exists for, on the first live session it met.

- **The mechanism is documented beside the leniency it qualifies, including where it is weaker
  than observing the wire directly.** task:12 wrote the comparison; task:13 corrected it. Strict
  mode's granularity is now stated exactly — one top-level field name in neither registry — with
  the five things it cannot detect listed beside it. The probe's row in the same table said
  "fails when: never; it has no model to be wrong", which was false: its failure modes are
  *independent* of the adapter's, not absent, and `probe.sh show` is a parser and a summary
  rather than raw evidence.

- **Nothing in this sprint changes what a recording claims, what the schema holds, or what the
  projection derives.** Held. task:12 added a capture field that reaches no record and a refusal
  path that writes nothing; task:13 changed documentation, comments, one shell tool, and one
  test file. `git diff src/` for task:13 contains no non-comment line.

### What the sprint found that it was not looking for

The instrument it was built to trust turned out to have a defect of its own. The probe appended
every payload to one shared NDJSON file with `cat >>`, and Claude runs matching hooks in
parallel: eight concurrent 512 KiB payloads through that implementation produce four lines in
total, two of which parse, leaving six of the eight payloads unrecoverable. It is now one
atomically completed file per hook invocation, with a concurrency regression test.

That is the sprint's rationale arriving one level up. The rationale said an adapter cannot audit
itself and needs an observation from outside; the sprint found that the outside observer also
needs checking, and that "it has no model to be wrong" is the same shape of sentence as "the
integration never sent this field".

### What this sprint deliberately leaves open

Everything its non-goals reserved. dragon:1, dragon:2, and dragon:3 are open. `permission_mode`
is still dropped and the schema decision dragon:1 argues for is still owed. **Interruption has
never been observed** — three sessions, three distinct reasons — and `PermissionDenied` has
never fired at all, so `tool_denied` remains reachable only by synthetic payload. None of those
is instrument work; each needs a session that has not been run, and a protocol that puts the
interruption in its own turn.
