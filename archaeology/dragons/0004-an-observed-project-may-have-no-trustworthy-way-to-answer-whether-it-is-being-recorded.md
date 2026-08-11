---
id: drg_01KZSJJGM41GWJV5Q0TY0X5D7W
sequence: 4
kind: dragon
status: open
created: 2026-08-11
---

# An observed project may have no trustworthy way to answer whether it is being recorded

## Context

Everything WitnessGlass is for depends on a recording existing, and a recording that was never
made cannot be reconstructed afterwards. "Is this session being recorded?" is therefore the one
question every observed project has to be able to answer, and it is the question this project has
never given a supported way to answer. Each consumer invents its own, and the answers are only as
good as whoever wrote them.

**Scope of what follows: measured 2026-08-11, macOS arm64, Claude Code 2.1.228, `witnessglass
0.1.0` installed via `cargo install`, in `symmetry-foundry` — a second external commissioning,
after cuecraft in log:1.**

### The designated source of truth declares itself inert

`scripts/arm.sh` states the design intent in its own header, and it is a deliberate choice rather
than an oversight:

> The sentinel this writes is not a second copy of "am I armed" — the settings file is already
> that, and a duplicate flag would only drift from it.

The settings file is the answer. Arming then produces it by `cp "$EXAMPLE" "$SETTINGS"`
(`scripts/arm.sh:120`) — a verbatim copy of `.claude/settings.witnessglass.example.json`, whose
`_comment` opens:

> EXAMPLE ONLY — THIS FILE IS NOT ACTIVE.
> Claude Code reads .claude/settings.json and .claude/settings.local.json.
> It does not read this file, so cloning the repository records nothing.

Every one of those lines is true of the template and false of the file it becomes. The artifact
nominated as the record of "am I armed" says, on its first line, that it is not.

This is not a downstream mistake. Both `.claude/settings.local.json.disarmed.*` files retained in
this repository — real armed configurations, installed by `arm.sh` and set aside by `disarm.sh` —
open with `EXAMPLE ONLY — THIS FILE IS NOT ACTIVE.` The defect reproduces here and has been
sitting in this checkout since 2026-08-03.

It also propagates. `symmetry-foundry` could not use `arm.sh` at all (idea:7) and wrote its own
`scripts/witnessglass-arm.sh` against a locally rewritten example file. The rewrite corrected the
binary path, added the `PATH`-resolution rationale, and documented the strict-JSON canary — and
kept `EXAMPLE ONLY - THIS FILE IS NOT ACTIVE.` as its opening line, then copied it into the active
configuration exactly as upstream does. A careful reimplementation reproduced the defect, which is
what makes it structural rather than a typo.

### The check consumers write cannot distinguish armed from template

`symmetry-foundry`'s operating contract opens with a mandatory startup check, run before any
substantive work:

```sh
ls .claude/settings.local.json && grep -q claude-hook .claude/settings.local.json \
  && echo "ARMED" || echo "NOT ARMED"
```

The string `claude-hook` appears in the inert template and in the installed copy alike, so this
tests for the presence of a string and reports it as a statement about recording. An unmodified
example file dropped into place prints `ARMED`. So does a configuration naming a binary that is
not on `PATH`. So does an armed repository in the session that armed it — the one session arming
is guaranteed not to cover.

Run on 2026-08-11 the check printed `ARMED`, and the repository genuinely was armed. It was right
by luck rather than by construction, and reading the file it had just approved produced a flat
contradiction: the file said it was not active. Two artifacts disagreeing, neither authoritative,
in the first thirty seconds of a session whose contract makes this check step zero.

### What actually settled it was out-of-band forensics

The question was answered by evidence the product does not offer:

- a recording in `.witnessglass/recordings/` whose **filename equalled the live session's id**;
- **record count growing 9 → 15** across two invocations of the local check script, which is a
  running recorder and cannot be anything else.

That is a real answer, and reaching it required knowing the recordings directory layout, the
session id, the filename convention, and that a growing file means live hooks. None of that is
knowledge an observed project should need, and log:1 already established the shape of the problem:
*"Once armed, the observed project needs to know nothing at all."* True of operating, false of
verifying.

The growth measurement is also self-implicating. log:1 measured that inspecting a recording from
inside its own session extends it by the records of the inspection. The 9 → 15 growth above **is**
that contamination — the most convincing available proof that recording is happening consists of
causing more of it. Any in-session answer to this question inherits that property.

### Two commissionings, two different answers, neither upstream

- **cuecraft** (log:1): hook configuration hand-authored, eight surfaces, none of `arm.sh`'s
  guarantees applying; completeness checked by `replay >/dev/null; echo $?`.
- **symmetry-foundry**: local reimplementations of both — `scripts/witnessglass-arm.sh` (recovering
  the self-test and displace-and-restore gates) and `scripts/witnessglass-check.sh` (recovering
  payload-silent checking) — plus a contract §0 ritual, plus its own `dragon:3` about sessions
  going unrecorded silently.

Both re-derived the same affordances because idea:6 and idea:7 remain parked. They differ in exit
codes, wording, failure messages, and how each states the mid-session-arming caveat. None lives
upstream, so a defect found in one is fixed in one. The observed project also ended up carrying the
recorder's own unknowns in its archaeology, which is knowledge in the wrong repository.

### Why this is a dragon and not idea:10

idea:6 through idea:9 are shaped features with known answers; this is a claim surface. The failure
mode here is not a missing answer but a **confident wrong one**, and the dangerous direction is the
false positive: a check reporting `ARMED` while nothing records produces exactly the silent hole
the recorder exists to prevent, with the aggravating property that somebody checked and was
reassured.

log:1 already recorded this defect shape once, in Scarp rather than in WitnessGlass — two builds
answering to the same version string, producing "a confident wrong conclusion, drawn from a real
transcript and corrected only because a human knew which checkout to look in." A settings file that
cannot say whether it is active is the same defect as a version string that cannot separate two
builds, and as a recording that cannot name the recorder that produced it.

## Question

Can an observed project obtain a trustworthy, standard answer to "is this session being recorded?"
— one that does not require knowing WitnessGlass internals, does not require reading the recording,
and cannot report armed when nothing is recording?

## Constraints

- **No hook-based guard is possible.** The hook is the recorder, so an unarmed session has nothing
  to warn with. `symmetry-foundry`'s dragon:3 states this; it is a property of the capture
  mechanism (§4), not of that project.
- **No daemon, watcher, or background process** (§6). Whatever answers this is foreground,
  on-demand, and dies with its invocation.
- **Cannot be answered by printing the recording** (§5, dragon:2). Payload-silent and exit-code
  shaped, as idea:6 already argues for completeness checking.
- **In-session verification contaminates what it verifies** (log:1). Any answer computed from
  inside the session it describes must say that it is part of what it measured.
- **Never take ownership of a settings file it did not write** (idea:7). An external project's
  `.claude/settings.local.json` may carry other tools' configuration.
- **"Armed" and "recording" are different claims** with different truth values in the same session.
  Arming never covers the session that arms. Anything reporting one as the other is the defect
  restated.
- **Claims expire per-hook and per-version** (dragon:2, pass 3). A settings file is configuration,
  not evidence that any hook fired on the installed Claude Code version.
- **Detection is not prevention.** A recording that was never made cannot be recovered, so noticing
  a gap afterwards is a strictly weaker outcome than not leaving one.
- Nothing here chooses the launcher fork. log:1 records it unresolved and it stays unresolved.

## Candidate direction

Not authorized. Recorded so the options are not re-derived later.

- **Stop shipping a self-negating source of truth.** Arming *generates* the installed configuration
  instead of copying the template byte-for-byte, so the active file states that it is active and
  carries what armed it, when, and with which build. The template keeps its `EXAMPLE ONLY` warning;
  the installed copy must not inherit it. Smallest piece, concrete defect behind it, fixes this
  repository as well as its consumers.
- **A `witnessglass status [DIR]` verb** answering the two questions separately by exit code: is
  this directory armed (a settings file installed, structurally a WitnessGlass hook configuration,
  naming a binary that resolves), and is a given session recorded (a recording exists for that
  session id and replays). Payload-silent, one-shot, no analysis of its own — the same discipline
  idea:6 applies to checking.
- **A supported integration contract**, single-sourced upstream: the startup check, the wording
  about mid-session arming, the privacy posture. Consumers cite it instead of reimplementing it, so
  a defect found once is fixed once.
- **The launcher fork** (log:1) remains the other shape: it makes the question moot for sessions it
  starts and answers "not recording" silently for every session started any other way.

## Resolution criteria

Resolved when:

- **No artifact WitnessGlass installs makes a false statement about its own status.** Specifically,
  the file `arm.sh` designates as the answer to "am I armed" no longer opens by declaring itself
  inactive — in this repository as well as in observed ones.
- An armed configuration is **mechanically distinguishable from an inert template**, by a property
  generated at arm time rather than copied, without knowledge of WitnessGlass internals.
- A supported affordance answers **"armed?" and "recorded?" separately**, payload-silent, by exit
  code, from an installed binary with no checkout present.
- That affordance is tested **against the false-positive direction**, not only the false-negative
  one. A template present but not installed, a configuration naming a binary absent from `PATH`,
  and a session that began before arming must each report not-recording.
- **What it cannot see is stated where it is used**: that armed is a claim about configuration
  rather than evidence a hook fired, that it expires on a Claude Code version change, and — for any
  in-session answer — that computing it added records to the recording it describes.
- At least one external project is commissioned using **only upstream affordances and upstream
  documentation**, writing no local stand-in scripts, and its archaeology carries no dragon about
  whether the recorder's armed state can be determined. Whether *people* remember to arm their
  clones is a discipline problem that legitimately stays with the observed project; whether the
  answer can be *trusted once asked* is this one.

**Escalated** if a session is ever found to have gone unrecorded in a repository whose startup
check reported armed. That would move this from a claim-surface defect to a demonstrated hole in a
corpus, and would make the generated-configuration fix urgent rather than tidy.
