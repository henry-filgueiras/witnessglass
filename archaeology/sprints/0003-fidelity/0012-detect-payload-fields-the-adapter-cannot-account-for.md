---
id: tsk_01KZ511T1RPW5GXE2GPZY83BA0
sequence: 12
kind: task
status: closed
sprint: spr_01KZ50ZMMD79D5FS65F4CTV9C2
created: 2026-08-03
closed: 2026-08-03
---

# Detect payload fields the adapter cannot account for

## Objective

Let the adapter be asked, on demand, to refuse any hook payload carrying a top-level field it can
neither record nor name as deliberately dropped — so that divergence between the wire and this
adapter's model of it is detectable rather than silent.

Not a hardening measure and not a default. A recorder that stops the day Claude adds a field is a
worse outcome than one that misses the field, and that trade is exactly why `HookPayload` ignores
unknown fields in the first place. This is a canary you arm for one session and then disarm.

## Acceptance criteria

- `witnessglass claude-hook --strict-json-validation` refuses a payload with an unaccounted-for
  top-level field, names the offending fields in the error, appends nothing, and exits 1 — which
  Claude documents as non-blocking for every hook this adapter subscribes to.
- `WITNESSGLASS_STRICT_JSON=1` is equivalent. A hook is spawned by Claude from a settings file
  that `arm.sh` writes from a fixed example, so a flag alone cannot reach one without hand-editing
  JSON a script owns.
- The set of modelled fields is not maintained by hand. A second list of "known keys" would have
  needed updating in the same commit that introduced the `duration` bug, by the same person.
- Fields dropped on purpose do not trip it. `cwd` arrives on every payload; if it fired the canary
  would fire on every hook of every session and be worth nothing. "Dropped on purpose" and "never
  heard of" must remain different facts, each with a reason recorded.
- The default path is unchanged, and a test says so.
- Documented in `docs/claude-adapter.md` beside the leniency it qualifies, including an honest
  account of where it is weaker than observing the wire directly.

## Result

Delivered as specified, in `e73e5e0`.

**Shape.** `HookPayload` grew `#[serde(flatten)] unmodelled: BTreeMap<String, Value>`, which
captures every top-level field the struct did not claim. That is what keeps the inventory honest:
the set of modelled fields *is* the struct, so adding a field removes it from the capture
automatically and no second list can drift. Beside it, `DELIBERATELY_UNRECORDED` names the seven
fields seen and dropped on purpose — `cwd`, `transcript_path`, `permission_mode`, `effort`,
`model`, `last_assistant_message`, `stop_reason` — each with its reason. Their union is a
complete, reviewable statement of everything this adapter has ever seen on the wire, and strict
mode is the alarm that fires when that statement goes stale.

**Serde cannot do this directly, and the flag name is slightly wrong about that.**
`deny_unknown_fields` is a compile-time attribute; there is no runtime switch to flip. Capturing
into a flattened map and diffing against the ignore list is better regardless: it reports *which*
fields were unaccounted for rather than only that some were. The flag keeps the name asked for.

**Calibration, on real evidence.** All 14 raw payloads the probe captured during the pass-2
hostile session were re-run through strict mode: 14 accepted, 0 refused. The canary is quiet
against Claude Code 2.1.220, which is what makes a future noise meaningful. A test freezes that
key set so the same check runs in CI.

**Would it have caught the original bug?** Yes — `duration_ms` was in neither the struct nor any
ignore list, so strict mode would have named it on the first payload of the first session, and a
test asserts this. Worth stating precisely, because `docs/claude-adapter.md` initially claimed the
opposite: that no strict mode could have caught it, since the adapter's model was the thing that
was wrong. That was false and was corrected before the commit. Strict mode compares the wire
against the model and reports the difference; it does not need the model to be right, only to be
*stated*.

The probe remains the stronger instrument, for a different reason than the one first written down.
It has no model to be wrong, reports every key rather than only the unaccounted ones, and costs no
refused records. Strict mode is cheap enough to run for a whole session and says *that* something
moved; the probe says *what*.

### Desire-path friction

**A task cannot exist outside an active sprint.** Both sprints were closed, so `scarp new task`
had nothing to attach to, and filing this one required commissioning sprint:3 first — a much
larger artifact carrying a goal, rationale, success criteria, and non-goals, for a piece of
housekeeping that needed a checkbox. Here that turned out well, because a real theme existed to
name. It would not have for genuinely isolated maintenance, and the pressure runs the wrong way:
the cheapest path is to skip tracking entirely, which is what the workflow exists to prevent.
The smallest useful affordance is somewhere for a task to live that is not a sprint — an
`--unsprinted` task, or a standing maintenance sprint that never closes — so the choice is "track
it" or "do not", rather than "track it" or "commission a sprint to hold it".

**The one section CLAUDE.md §8 requires is the one Scarp will not write.** §8 says friction goes in
the active task's `## Result`. `scarp new --body-file` refuses a body containing `## Result`,
correctly, since Scarp owns the template and a task's sections are `Objective` and
`Acceptance criteria`. But `scarp close` does not add the section either, and no command appends
to an artifact. So the section every task is contractually obliged to carry can only arrive by
`cat >> file.md`, hand-matching a heading level against a template nothing validates. That is how
this Result got here, and how three dragons were extended earlier in the same round. `scarp
doctor` passed each time, so nothing was corrupted — but doctor is checking work the tool declined
to do.

The affordance is either half: `scarp close task:N --body-file <path>`, filling `Result` in the
same write that closes the task, or a general `scarp append <ref> --body-file <path>` that knows
which sections a collection has. The second is more useful, since dragons and sprints accumulate
follow-ups the same way and hit the same gap.
