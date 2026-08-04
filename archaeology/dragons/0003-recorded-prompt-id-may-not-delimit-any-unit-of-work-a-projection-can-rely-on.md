---
id: drg_01KZ2B63TES6DCPZS2QR81ZT74
sequence: 3
kind: dragon
status: open
created: 2026-08-02
---

# Recorded prompt_id may not delimit any unit of work a projection can rely on

## Context

decision:4 added `context` to the raw stream envelope holding "only causal identifiers the
integration actually supplies", and `prompt_id` was the first of them. The reasoning was
sound and deliberately minimal: record what Claude hands over, invent nothing, and defer the
question of whether a causal overlay can be built until a real session had been recorded.

task:4 recorded one. `prompt_id` arrived populated on 233 of 234 records — the single
exception being `session_started`, which carried no `context` at all, exactly as the
documentation predicts for a payload that precedes the first input. On the face of it that is
a clean result: the field the schema reserved for causal context is not empty.

Then the distribution is looked at. Across a 17-minute session containing 82 tool calls,
one subagent, and 234 records, only **two** distinct `prompt_id` values appear. One covers
232 records — effectively the entire working session, including every tool call made by both
the parent agent and the subagent. The other appears on exactly one record, the
`session_ended`.

Nothing in the recording explains that shape, and the recording is structurally incapable of
explaining it, because `UserPromptSubmit` is deliberately not captured (task:6, decision:4).
There is therefore no prompt event anywhere in a WitnessGlass recording for a `prompt_id` to
refer to. The identifier is populated and **unanchored**: it is a key with no record on the
other end of it.

That leaves at least these readings alive, and the recording chooses between none of them:

- the session genuinely was one user turn, and `prompt_id` delimits a turn;
- `prompt_id` changes on some events and not others, so what it delimits is not a turn at all;
- it is scoped to something else entirely — a request to the model, a conversation segment, an
  internal lifecycle boundary — that happens to look turn-shaped in a single-prompt session;
- the value on `session_ended` marks a real second unit of work, or marks the exit machinery
  itself, and these are different claims.

The subagent makes this sharper rather than easier. Its 27 tool calls carry a distinguishable
`context.agent_id`, which is a genuine, delivered attribution — but they carry the *parent's*
`prompt_id`, unchanged. So the one identifier that does segment the recording usefully is
`agent_id`, and the one intended to carry causal context does not vary at the point where the
causality visibly branches.

The danger is not that the field is useless. It is that it looks useful. A `prompt_id` on
nearly every record is exactly the shape a projection reaches for when it needs a unit of
work: group tool calls by prompt, count turns, measure per-turn duration, render a timeline
segmented by prompt, say "in this turn the agent did N things". Every one of those is a claim
about a unit whose definition this project has never established and cannot establish from a
recording alone. Building any of them would be the silent promotion decision:2 forbids,
committed not by fabricating a value but by attaching a meaning to a delivered one.

This is a different dragon from the two already open. dragon:1 asks what is *observable*.
dragon:2 asks what is *safe to share*. Neither asks what an observed identifier *means*, and
the answer to that is a precondition for every derived projection in section 3 of `CLAUDE.md`.
The same question applies, with less evidence behind it so far, to `agent_id`, `agent_type`,
and `tool_use_id`: `tool_use_id` is the one identifier whose semantics the project has
actually tested, through 82 correlated request/completion pairs, and it is the exception
rather than the rule.

## Question

What does `prompt_id` actually delimit, what unit of agent work may a projection legitimately
build on it, and how would WitnessGlass know that from evidence rather than from assumption?

## Constraints

- No meaning may be attached to a delivered identifier without evidence for that meaning. A
  populated field is evidence that the integration supplied a value, and nothing more.
- Raw recordings are not changed to answer this. `context.prompt_id` is already recorded
  exactly as delivered, and that stays true whatever is concluded. Any unit of work is a
  derived projection and must be rebuildable from the raw stream (`CLAUDE.md` §3).
- Capturing `UserPromptSubmit` would give the identifier something to refer to, and would also
  put user prompt text — the most sensitive surface in a session — into the raw stream. That
  is a dragon:2 decision, not a free one, and it must not be taken merely to make this
  question easier.
- Whatever is concluded must degrade honestly across versions and hosts. `prompt_id` semantics
  are Claude's, are undocumented beyond "absent until the first input", and may change without
  notice. A conclusion that only holds for one version has to say so.
- "We cannot tell what this delimits" is a supported, publishable answer, and is preferable to
  a plausible unit of work that nobody has verified.
- No projection, timeline, or summary may segment by `prompt_id` until this is settled. That
  includes describing a recording informally as containing "N turns".

## Candidate direction

Measure before deciding anything, and keep the measurement cheap.

The obvious experiment is a recorded session with several deliberately distinct user turns,
which would say immediately whether the value changes per turn. That is one recording and it
answers the first-order question. A second recording under `--resume`, and one containing a
compaction, would say whether the identifier is stable across the boundaries a session can
cross. None of this requires a schema change, a new hook, or any code.

If `prompt_id` turns out to delimit a turn reliably, the honest output is a documented,
version-scoped statement of that fact in the adapter's fidelity section — not a projection
built on it, yet. If it turns out not to, the equally honest output is that `context.prompt_id`
remains a recorded opaque identifier with no defined unit, and derived views segment by
something else or by nothing.

The tempting shortcut — capture `UserPromptSubmit` so the identifier has an anchor — is
deliberately *not* the first move. It trades a large privacy surface for a question that a
plain observational experiment can answer first, and it should only be considered after the
observation has been made and found insufficient.

## Resolution criteria

This dragon is resolved when:

- `prompt_id`'s behaviour has been observed across at least a multi-turn session, and the
  observation states which Claude Code version and host it holds for.
- A written statement exists saying what `prompt_id` is known to delimit, what it is not known
  to delimit, and what a reader may and may not conclude from two records sharing one — with
  the negative result treated as a complete answer if that is what the evidence shows.
- The adapter's fidelity documentation carries that statement where a user encounters it, at
  no greater strength than the evidence supports.
- Any decision about capturing `UserPromptSubmit` is made explicitly, on the basis of that
  measurement and of dragon:2, rather than as a side effect of wanting a nicer projection.
- No projection in the codebase segments work by `prompt_id` unless this dragon licensed it,
  and the same question has been asked of every other identifier a projection wants to lean on.

## Observation: two real turns, two `prompt_id` values, and that is still not a licence to segment

Appended 2026-08-03 from the hostile session recorded to `docs/hostile-recording.md`.
**Scope: Claude Code 2.1.220, macOS arm64, one session, two turns submitted as two separate
prompts, 40 records.**

This is the first-order observation the dragon asked for, and it is clean:

| records | `prompt_id` | what was submitted |
| --- | --- | --- |
| 2–29 | `faf9b390…` | turn 1 (`/hostile-1`) |
| 30–40 | `ec6c59fa…` | turn 2 (`/hostile-2`) |

One value per turn, no overlap, the boundary exactly where the human pressed enter the second
time. Record 1 (`session_started`) carries no `prompt_id` at all, matching the documented "absent
until the first input". The subagent's own tool calls carry turn 1's value, so a `prompt_id` spans
the parent and its subagent rather than distinguishing them.

**What this licenses: `prompt_id` changes at a turn boundary.** That is a real result and it is
worth having.

**What it does not license: segmenting a recording by `prompt_id`.** The first-contact session
produced two distinct values across what its operator understood to be a single turn — one on 232
records, one on `session_ended` alone. Both observations are consistent with a rule like "a new
prompt gets a new id, and some other events get their own", and that rule permits N values for one
turn. Changing at a turn boundary is **necessary** for a turn identifier and not **sufficient**,
and nothing observed so far distinguishes `prompt_id` from an identifier that changes more often
than turns do.

So the second resolution criterion can now be written honestly, and it is a negative:

> A reader may conclude that two records with **different** `prompt_id` values were not produced
> by the same submission. A reader may **not** conclude that two records sharing one belong to the
> same turn, that a recording contains as many turns as it has distinct `prompt_id` values, or
> that any span between changes is a unit of work. Observed on Claude Code 2.1.220, macOS arm64,
> across two sessions: one where two values appeared inside a single turn, and one where each of
> two turns had exactly one.

That is enough to keep the CLAUDE.md §6 condition (no segmentation by `prompt_id`) standing on
evidence rather than on caution, and enough for the adapter's fidelity section to say something
useful. It is not enough to resolve the dragon: the criteria also require the statement to reach
the adapter documentation, and require the `UserPromptSubmit` question to be decided explicitly.

**A candidate anchor appeared, and it should be treated with suspicion.** `SubagentStop` fired
once near the end of each turn, with an `agent_id` seen nowhere else and an empty `agent_type` —
see the corresponding note in dragon:1. It is tempting as a turn delimiter precisely because it
sits where a turn ends. Two occurrences, no documentation, no stated semantics, and adopting it
would mean inferring a unit of work from an event that does not claim to mark one. That is the
same mistake as segmenting by `prompt_id`, wearing a different hat.

## Pass 3: the turn-boundary result reproduces, and the `session_ended` observation does not

Appended 2026-08-04. **Scope: Claude Code 2.1.221, macOS arm64, one session, two turns submitted
as two prompts, 39 records.** Pass 2 ran on 2.1.220; this is a different version.

The turn-boundary behaviour reproduces exactly:

| records | `prompt_id` | what was submitted |
| --- | --- | --- |
| 1 | *(none)* | `session_started` |
| 2–29 | `49a7bed5…` | turn 1 (`/hostile-1`) |
| 30–39 | `ead8e077…` | turn 2 (`/hostile-3`) |

One value per turn, boundary where the human pressed enter, absent before the first input, and
spanning the parent agent and its subagent alike. Two sessions, two versions, same behaviour. The
statement written into this dragon after pass 2 stands unchanged, and now has a second
independent observation behind it.

### The `session_ended` sub-observation does not reproduce, and it was load-bearing

First contact reported two distinct `prompt_id` values across a session its operator understood as
a single turn: one covering 232 records, and **one on `session_ended` alone**. That second value
was the entire basis for treating `prompt_id` as an identifier that changes more often than turns
do — the reason "changes at a turn boundary" was called necessary but not sufficient.

Pass 3's `session_ended` carries **turn 2's** `prompt_id`, not a value of its own. So the two
sessions disagree about the one record that mattered.

Both recordings were re-read to check this rather than taken from the earlier write-up, and the
comparison is tighter than expected:

| | first contact | pass 3 |
| --- | --- | --- |
| `session_ended` sequence | 234 | 39 |
| `reason` | `prompt_input_exit` | `prompt_input_exit` |
| `prompt_id` on it | `8c59ba18…`, **carried by no other record** | `ead8e077…`, **shared with turn 2's other records** |

**The two sessions ended the same way.** `reason` is identical, so "it depends on how the session
terminated" is not available as an explanation — which was the reading that would have made this a
harmless quirk. Something else differs.

The most economical account left is that first contact had a further prompt context that produced
no tool calls at all: a submission answered in prose, or an input begun and abandoned, with
`session_ended` carrying it because it was simply the current one. That is a softer form of "it
was more than one turn" — not two turns of work, but a turn that left no tool evidence.

If that is right, `prompt_id` is per-submission in all three sessions, the in-turn change first
contact appeared to show never happened, and this dragon's central worry is smaller than it looks.
It would also mean **a recording can contain a turn that is invisible within it**, which is its
own finding and not a comforting one.

The alternatives are that the behaviour changed between 2.1.220 and 2.1.221, or that
`session_ended`'s identifier follows a rule nobody here has characterised.

This project cannot choose between them, and the reason is worth stating plainly: first contact
was not recorded to a protocol, the operator's prompt count was never written down at the time,
and `UserPromptSubmit` is not captured, so no recording can say how many submissions it contains.
**The evidence that would settle it existed only while it was being generated.**

### Where this leaves the criteria

The negative statement written after pass 2 still holds and is what the adapter documentation
should carry:

> A reader may conclude that two records with **different** `prompt_id` values were not produced
> by the same submission. A reader may **not** conclude that two records sharing one belong to the
> same turn, that a recording contains as many turns as it has distinct values, or that any span
> between changes is a unit of work.

What has changed is the reason it holds. After pass 2 the counterexample was first contact's
apparent in-turn value change; that counterexample is now itself in question, and the honest
position is that the second sentence rests on an ambiguity rather than on a clean contradiction.

It should not be weakened on that account. An identifier whose behaviour at `session_ended`
differs between two sessions that ended identically is not one to segment by, and the ambiguity
argues for caution rather than against it. But the footing has moved and a future reader should
know which stone it is standing on.

**Resolving this is cheap and nobody has done it.** It needs the operator's submission count
written down beside the recording, at the time, as a deliberate separate act — because the
recording provably cannot supply it. Every future pass should do this, and it should be a numbered
step rather than an instruction in prose, since two passes have now demonstrated that the
unscripted parts are the ones that get missed.
