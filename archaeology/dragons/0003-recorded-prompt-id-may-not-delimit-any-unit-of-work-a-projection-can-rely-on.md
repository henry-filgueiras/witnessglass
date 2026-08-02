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
