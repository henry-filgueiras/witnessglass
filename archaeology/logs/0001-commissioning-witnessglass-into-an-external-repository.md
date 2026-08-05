---
id: log_01KZA4FTMNWR9S33DE95Y3PPVF
sequence: 1
kind: log
created: 2026-08-05
---

# Commissioning WitnessGlass into an external repository

On 2026-08-05, WitnessGlass was commissioned into **cuecraft** — a TypeScript project with no
relationship to this one — as the first attempt to instrument a repository that is not this
repository. The exercise was run as a usability probe as much as an installation: the question
was not only whether it could be made to work, but what an external project has to know about
WitnessGlass internals in order to be observed by it.

This log records what was measured and what was not. Nothing here was fixed; this round
collected evidence only. The actionable findings are idea:6, idea:7, idea:8, and idea:9. What
remains below is the part that is not a feature request.

## What was installed, and what worked

`cargo install --path <checkout> --locked` worked on the first attempt — release profile, 1.77s,
executable `witnessglass` on `PATH`. `publish = false` does not obstruct it. This is worth
recording because nothing in the README says the crate is installable, and `publish = false`
alongside crate publication as a standing non-goal reads as though it is not.

The installed binary passed `arm.sh`'s three gates, re-run by hand against a synthetic
`SessionStart` payload: exit 0, **zero bytes on stdout**, recording written. `replay` round-tripped
it. `view` projected it, bound loopback, and served 200 with the per-launch capability and 404
without it — from the external project's working directory, against an installed binary, with no
checkout in sight.

So the kernel, the adapter, and the viewer are all reachable from outside this repository. What
is not reachable is the machinery around them.

## What had to be hand-authored, and what that cost

The hook configuration. `scripts/arm.sh` is bound to its own checkout and the committed example
names the binary through `${CLAUDE_PROJECT_DIR}`, which in an observed project points at a path
that does not exist — so cuecraft's `.claude/settings.local.json` was written by hand, eight hook
surfaces, with none of `arm.sh`'s guarantees applying to it. That is idea:7. One guarantee was
recovered by hand — the synthetic-payload self-test, which is currently inline in `arm.sh` and
reachable no other way; exposing it as a verb is a small point not filed as its own idea.

Two smaller things had to be invented on the far side: how to find a recording (idea:8) and how
to check one without printing it (idea:6, unreachable with an installed binary).

The shape of the result is worth stating plainly. **Once armed, the observed project needs to
know nothing at all** — `cd cuecraft && claude` is the whole interface, because Claude reads the
settings file itself. The entire external-project gap is in arming, not in operating. That is a
narrower problem than it looked at the start of the round, and it is the reason idea:7 is scoped
to arming rather than to a general integration surface.

## What has NOT been validated

**No Claude session has been recorded in cuecraft.** Everything above was measured with synthetic
payloads and an already-running, uninstrumented session. That the hooks fire at all, that all
eight surfaces fire, and that a real external session produces a complete `session_started` →
`session_ended` recording are all unmeasured, and the commissioning deliberately stopped at that
boundary rather than simulating it.

One documentation-derived assumption is load-bearing and unmeasured with it: that Claude's exec
form resolves `command` on `PATH`. That is read from the hooks reference (re-read 2026-08-05),
and this project has been burned before by believing a reference over a probe — the `duration_ms`
episode cost two sprints. If the first instrumented session records nothing, this is the first
thing to suspect.

## What a recording could not say about the project it observed

The provenance audit this round called for, stated as findings rather than as work:

Present, per record — `schema_version`, `session_id`, `sequence`, `recorded_at`, and
`provenance` as `{channel, adapter, mechanism}`. Session start and end times are derivable from
the boundary records; `session_ended` carries a reason.

Absent — the observed repository, its revision, the WitnessGlass version, the WitnessGlass
revision, and the Claude Code version. `cwd` and `permission_mode` arrive and are deliberately
dropped, each with a stated reason.

A recording therefore identifies **which session** but not **which project, which commit, or
which recorder**. Inside this repository that was invisible: there was only ever one project, and
it was this one. Instrumenting a second project makes it structural — the recordings live in
cuecraft's `.witnessglass/`, and the directory they sit in is the only thing linking them to the
work they explain.

idea:9 covers the narrow half (a binary that can name its own build). The other half is not a
feature request, and is below.

## Architectural questions this raised, which are not being answered here

**1. Session-scoped provenance collides with the privacy posture.** Recording which project and
which revision a session observed wants a once-per-recording recorder-channel record, not fields
repeated on every event. But repository identity *is* filesystem identity unless a different
granularity is chosen deliberately, and §5 and dragon:2 are why `cwd` is dropped in the first
place — dragon:2 also found `cwd` arriving inside error strings, where dropping the field does
not help. "Record the repo" and "do not record the operator's filesystem" are the same decision
viewed from two sides, and it wants a decision artifact before it wants code.

**2. Arming and launching are different products.** idea:7 sketches `witnessglass arm <DIR>`:
persistent configuration written into someone else's repository, with an obligation to give it
back unchanged. The alternative shape is a launcher — `witnessglass claude`, or
`witnessglass run -- claude` — that configures one ephemeral process and leaves nothing behind.
They differ in what can go wrong: arming can corrupt a settings file it does not own and can be
left armed by accident; launching cannot, but observes only sessions started through it, and
silently observes nothing when someone runs `claude` directly. One data point does not choose
between them.

**3. Where an external project's recordings belong.** Today's implied policy puts them in the
observed repository's `.witnessglass/`, which is what cuecraft was commissioned with. The
consequence is that every observed project needs a gitignore entry and has to know the directory
name — WitnessGlass knowledge in a project that should need none. A per-user store keyed by
project would invert that, at the cost of making recordings harder to find and easier to
accumulate unnoticed. This was decided by default rather than on purpose, which is the reason it
is written down.

## Notes on the record itself

This log is the first artifact in the `logs` collection in this repository, written with a
pre-release Scarp. Published Scarp 0.2.0 — which CLAUDE.md §7 names as current and which
`scripts/check.sh` invokes — cannot read it: `scarp list logs` reports an unknown collection, and
`scarp doctor` silently checks 54 artifacts where the pre-release build checks 55. The gate stays
green either way, and this repository already carries `maintenance:1` in the same position, so
the trade was already made and is noted here rather than introduced.

The friction findings above were collected under an explicit instruction to gather evidence
without modifying this repository, and were written here afterwards with authorization. cuecraft
records none of them: its own archaeology carries only the decision that it is observed
(cuecraft's `decision:7`), not the defects of the thing observing it.
