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

## Addendum — the first recorded external session (2026-08-05)

The validation gap left open above has been closed. A real Claude session in cuecraft was
recorded, and this addendum was written from inside that same session. It is a later append,
not a revision; nothing above has been altered.

**The `PATH` assumption holds.** This log named one load-bearing, documentation-derived belief —
that Claude's exec form resolves `command` on `PATH` — and predicted that if the first
instrumented session recorded nothing, that belief would be the first thing to suspect. It
recorded. The exec form (`command: "witnessglass"` plus `args`) resolved on `PATH` on macOS,
with no machine-specific path anywhere in the settings file. The failure this braced for did not
happen. Worth noting for idea:5: the prediction is legible as predating its result only because
the two sit in the same artifact in written order, which is precisely the property idea:5 wants
to stop depending on good manners.

**Five of eight surfaces fired; three remain unmeasured.** Observed in a single session:
`SessionStart` (`session_started`, source `startup`, sequence 1), `PreToolUse`, `PostToolUse`,
`PostToolUseFailure`, and `SubagentStop`. `PermissionDenied` was never exercised — the session
ran in an auto permission mode and nothing was denied. `SessionEnd` cannot be observed from
inside the session whose end it marks, so a complete `session_started` → `session_ended`
recording is still unvalidated. `SubagentStart` never fired at all, which is the next finding.

`duration_ms` was present and plausible throughout — 403 ms for a trivial `echo`, 23 ms for a
failing `jq`.

### Evidence for dragon:1 — `SubagentStop` fires with no `SubagentStart`

Twice, once at the end of each completed turn, a `subagent_stopped` record was written carrying
a populated `agent_id` and an **empty-string** `agent_type`. No subagent was spawned at any
point in this session, and no `subagent_started` record exists anywhere in the recording. The
two `agent_id` values differ from each other.

The adapter therefore emits a stop for something it never saw start, and cannot say what kind of
thing it was. Whether these are main-loop turn boundaries surfacing on the subagent hook, or
genuine internal agents that `SubagentStart` does not cover, the recording does not distinguish
— and that is dragon:1 in its concrete form rather than its anticipated one.

One part of this is a WitnessGlass choice rather than a Claude one. An `agent_type` of `""`
renders a missing value as a present field. Under §6 an absence should render as an absence, so
either the field should be omitted or the absence should be explicit.

### Evidence for dragon:3 — `prompt_id` under-counts user input

A user message arrived **mid-turn**, after tool calls had begun and before the turn ended, and
was answered inside that same turn. It minted no new `prompt_id`; the turn continued under the
existing one. Across the session, four user messages produced three `prompt_id` values.

This sharpens dragon:3 from "`prompt_id` may not delimit a unit of work" to something stricter:
`prompt_id` is not in bijection with user messages, so a projection that counted them would
under-report user input rather than merely mis-segment it. The §6 prohibition on segmenting by
`prompt_id` is the right posture, and this is a second, independent reason for it.

### Evidence for dragon:2 and idea:6 — inspection is not free

The unredacted posture is literal, as documented: full command text and full `stdout`/`stderr`
land verbatim. The failing-`jq` record additionally carries filesystem paths inside its error
string — the same shape dragon:2 already found for `cwd`, and the same lesson, that dropping a
field does not remove the value from prose that quotes it.

Less anticipated: **reading a recording extends it.** Inspecting the file with shell tooling
grew it by roughly twenty records of inspection, interleaved with the records being inspected.
There is no way to check a recording from inside the session that produced it without becoming
part of what you are checking. idea:6 is framed as a convenience — checking completeness without
printing — but the observer effect makes it closer to a correctness affordance: a verb that
reads a recording without transiting the tool surface would not contaminate what it reads.

### Friction encountered writing this addendum

`log:1` could not be appended with Scarp: **no Scarp version exposes an append or amend verb.**
The command surface is identical in the published 0.2.0 on `PATH` and in the pre-release build
in a sibling checkout — `init`, `new`, `proposal`, `list`, `show`, `doctor`, `close`, `reopen`,
`adopt`, `reject`, `fortune`, `resolve`, `completions` — and adding prose to an existing artifact
is not among them. This is exactly local idea:4, encountered a second time and from a different
direction: idea:4 was filed about amending a *closed* artifact, whereas `log:1` is open and still
unwritable.

The workaround was to edit the Markdown body directly, touching no front matter, sequence, slug,
or path — the things §7 reserves to Scarp. The pre-release `doctor` accepts the result. But it is
a hand-edit, and nothing in the record marks it as one.

**Two builds answer to the same version string.** A first pass at this section wrongly reported
the unreachable `logs` collection as a second, independent blocker. It is not one: the binary on
`PATH` is the published 0.2.0 that cannot see the collection, while the pre-release build that
reads and validates it sits in a sibling checkout. Both report `scarp 0.2.0`. They differ by two
collections and by what `doctor` counts — 58 artifacts against 60 — and nothing in the CLI
distinguishes them, with the narrower one installed on `PATH`.

This is idea:9's problem — a binary that cannot name its own build — surfacing in Scarp rather
than in WitnessGlass. It is worth recording in that form because the failure was not a stuck
command but a confident wrong conclusion, drawn from a real transcript and corrected only
because a human knew which checkout to look in. A version string that cannot separate two
builds is the same defect as a recording that cannot name the recorder that produced it.
