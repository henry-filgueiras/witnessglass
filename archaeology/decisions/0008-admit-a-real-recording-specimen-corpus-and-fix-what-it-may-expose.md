---
id: dec_01KZA7T44H8Q7A4B67T40RX0Y1
sequence: 8
kind: decision
status: accepted
created: 2026-08-05
---

# Admit a real-recording specimen corpus and fix what it may expose

## Context

sprint:16 ran an exposure study over real recordings drawn from **two repositories** — this one and
cuecraft, after log:1 recorded the first instrumented external session. Which recordings exist, where
they live, which project produced them, and which are admissible as experimental specimens is now
load-bearing for every result that cites them, and it was recorded nowhere: it survived only inside one
task Result, as prose.

Six recordings existed at the time of that study; four were admitted. That inventory was reconstructed
by hand from the filesystem, which is exactly the kind of knowledge `CLAUDE.md` §7 calls durable and
this repository was not keeping.

**A second problem the inventory must not create.** An index of recordings is one query away from being
an index of *what is in* those recordings. Every round since task:19 has held a hygiene bar — mechanically
derived quantities may be reported, contents may not — and an inventory is precisely where that bar
would erode, because listing a specimen invites describing it.

## Decision

**A specimen is admitted by this decision, and nothing is a specimen until it is.**

### Admitted specimens

**Observational — real recordings, no ground truth of any kind.**

| specimen | origin | events (observed) | first used | why admissible |
|---|---|---|---|---|
| `8b68dece` | witnessglass | 234 (169) | task:14 | the first-contact development session; the largest observational specimen |
| `57f18ff9` | witnessglass | 39 (32) | task:19 | a hostile-protocol run; independent task from `8b68dece` |
| `f5c18299` | witnessglass | 40 (33) | task:19 | a second execution of that runbook — a **known sibling** of `57f18ff9`, admitted only as a positive control and never as an independent sample |
| `7d95c414` | **cuecraft** | 106 (77) | task:26 | the first instrumented session in an external repository (log:1); the only specimen from a second project |

**Controlled — synthetic, structure decided in generator constants before any detector existed.**

| specimen | origin | first used | role |
|---|---|---|---|
| `synthetic-behavioral-oracle` | committed fixture | task:14 | the legible oracle; planted figure with known boundaries |
| `synthetic-behavioral-oracle-sparse` | committed fixture | task:15 | the sparse stress case at a real-session density |
| gauntlet families | generated per trial | task:22 | eight adversarial families, seeds recorded |
| adversarial families | constructed in the representation | task:25 | ten families built against inverse-frequency weighting |

### Excluded, and why

| recording | origin | reason |
|---|---|---|
| `c3afa0ca` | witnessglass | one `session_ended` record; no vocabulary and no span |
| `6a8a02cc` | cuecraft | same |
| `.witnessglass/probe/raw-hooks.ndjson` | witnessglass | raw hook payloads, not a recording; a fidelity probe with its own contract |

### What may be surfaced

On any public or report surface, for an observational specimen: the **opaque session identity** already
in use, event and record counts, span in seconds, vocabulary size, delivered mark counts and
frequencies, and any numerical statistic derived from candidates. Those are the quantities every round
since task:19 has published and they remain publishable.

### What must not be surfaced

Prompt text, response text, command text, tool output, file contents, absolute or repository-relative
paths, host or user identity, and any excerpt of a payload — **including to make evidence easier to
inspect**. That exception is the one that would swallow the rule. A recording is not committed, copied
into tracked storage, or reproduced, from either repository.

**This inventory is itself bound by that list.** It names specimens and counts and says nothing about
what any of them contains.

### The epistemic line

Observational specimens have **no known true motif boundaries**. They can measure exposure, behaviour,
and distribution; they cannot adjudicate whether a detector is correct. Controlled specimens can, and
only within the structure their generators planted. Any surface that reports an observational result
states this, and `tests/envelope.rs` asserts the evidence page continues to.

## Consequences

- A round adding a specimen amends this decision rather than describing the addition in a Result.
- The runbook-sibling dependence between `57f18ff9` and `f5c18299` is recorded once, here, instead of
  being re-derived by each round that uses them.
- **One observation is banked without interpretation.** The two largest observational specimens —
  `8b68dece` (witnessglass) and `7d95c414` (cuecraft), from unrelated projects — place **0.3787** and
  **0.3766** of their observed events on the same delivered mark, `v2:tool_requested/Bash`. Verified
  from source in task:26. It is an observation about two recordings. It is not a universal, a workflow
  label, a causal claim, a detector result, or evidence that any figure recurs, and nothing may cite it
  as one.
- The corpus is four observational specimens from two projects. That is an envelope, not a distribution,
  and every conclusion drawn from it inherits that limit.
