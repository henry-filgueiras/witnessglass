---
id: spr_01KZ50ZMMD79D5FS65F4CTV9C2
sequence: 3
kind: sprint
status: active
created: 2026-08-03
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
