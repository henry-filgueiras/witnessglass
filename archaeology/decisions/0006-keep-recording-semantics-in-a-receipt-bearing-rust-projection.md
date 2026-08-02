---
id: dec_01KZ2EBVHQBFZG6Y5FZ6FEJMSV
sequence: 6
kind: decision
status: accepted
created: 2026-08-02
---

# Keep recording semantics in a receipt-bearing Rust projection

## Context

sprint:2 needs one closed recording to be readable in a browser. That requires deciding, before
any of it is built, where the meaning of a recording lives — because there are exactly two
plausible answers and only one of them is compatible with this project's invariant.

The tempting answer is that the browser reads the recording. NDJSON is trivially parseable in
JavaScript, the payloads are already JSON, and a page that fetches the file and renders it needs
no intermediate layer at all. That answer is wrong for the same reason task:7 kept a parser out
of `check-recording.sh`: it produces a second implementation of what a recording says. Two
implementations are two opinions, and the moment they disagree — about a truncated tail, about
which outcome belongs to which request, about whether an absent field means "no" or "not
supplied" — the recording has stopped being a single account of a session.

The projection also has to survive a specific temptation that decision:5 named as the one most
likely to be broken by accident: filling a gap with a plausible value because the gap is
inconvenient to render. task:4 supplies the concrete gaps. `duration_ms` arrived zero times
across 82 completions. `parent_agent_id` arrived zero times across three subagent records. A
`subagent_stopped` arrived with no matching start. Files were changed by a shell command with no
mutation event anywhere. Each of those is a place where a renderer could produce something
plausible and be confidently wrong.

task:8 built the projection and its tests first, so that this decision describes a boundary that
exists rather than one that is intended.

## Decision

**The raw recording and its replay remain the authority.** Validation, schema dispatch, sequence
and session invariants, and the corrupt-versus-truncated distinction stay in `replay`, unchanged
by this decision. Corruption fails there and never reaches a projection. Canonical order is
append order and nothing sorts by timestamp.

**One projection layer sits between replay and every rendering: `witnessglass::inspection`.** It
is a pure, deterministic function of an already validated `Replay`. It reads no file, consults no
clock, and takes its input by shared reference, so it cannot mutate what it derives from. It
borrows the raw records rather than copying them, which makes "never rewrites raw evidence" a
property of the types rather than a discipline. It preserves the schema version including `None`,
the tail state, every raw record, exact append order, every sequence number, and timestamps as
descriptive metadata.

**Every derived entity carries receipts.** A correlation, a cardinality classification, an
anomaly, an aggregate, a coverage summary, a timestamp extremum: each carries the raw sequence
numbers supporting it. A derived claim that cannot produce its receipts is asserting rather than
deriving, and the model has no way to express one.

**A negative claim carries the scope it was reached in, not just an empty list.** An empty match
list alone is not evidence of absence. Counts travel as a matching-record list *plus* the
examined population, which is either a complete recording or the valid prefix of a truncated one.
That preserves the difference between "no matching record in this complete recording", "no
matching record in the valid prefix of this truncated recording", and any claim about what
happened outside recorded evidence — the third of which the model cannot state at all. Every
count means "records observed" and never "events that occurred", which is how "zero `tool_failed`
records" gets said without saying "nothing failed".

**Correlation is schema-specific and never flattened.** v1 correlates through `tool_call_id`; v2
correlates through `tool_use_id`; the two are tagged so that ids spelled identically under
different schemas are different keys. v1's claim that a call was witnessed *beginning* and its
succeeded/failed vocabulary stay v1's; v2's request, success, execution failure, and permission
denial stay four distinct things, and requested input stays separate from effective input. A
request is never read as proof of execution.

**Nothing is greedily paired.** Evidence is grouped by correlation id and classified honestly:
intent only, opening without outcome, outcome without opening, exactly one of each, duplicates on
either side, and semantically conflicting outcomes. Only the unambiguous one-to-one case may be
described as a paired lifecycle, and even then it is a correlation between two records — exposed
as two canonical positions, never as elapsed time, execution duration, nesting, or causal
containment. Divergent delivered fields, such as two tool names for one id, are kept as delivered
values with receipts rather than resolved by choosing one.

**Reported intent is correlated, never fused.** It remains a separate record on a separate
channel with its own receipts. No intent is reconstructed from a command, tool name, payload
description, path, temporal proximity, or result.

**Three agent identities stay structurally separate:** the current agent a record was delivered
from (`context.agent_id`/`agent_type`), the child agent a subagent lifecycle event is *about*
(the event's own `agent_id`), and a parent identity, which exists only when the event delivered
one. A subagent event's child id is never used as the emitter's identity. Absent identity is
"not supplied", never "root" or "main". A v1 record is "not representable" — its envelope has no
causal context at all, which is a different fact from asking and getting nothing. Where one id
arrives with inconsistent types, every delivered value is retained and the disagreement is
exposed.

**Survivable irregularities are evidence, not errors.** Missing or duplicated session boundaries,
unmatched requests and outcomes in either direction, conflicting outcomes, unmatched subagent
boundaries, and a truncated valid prefix all project. None of them fails.

**`prompt_id` survives as raw context and as a field-presence count.** It defines no group, turn,
session segment, or unit of work while dragon:3 is open.

**A derived claim is a projection-level object, not a new raw channel.** `Channel` keeps exactly
`reported`, `observed`, and `recorder`. Those describe how a record reached the recording; a
derived claim never reached the recording. Adding a fourth value would blur the raw provenance
the projection exists to carry forward. Derivation is orthogonal to channel provenance, and the
projection is derived by construction.

**Rust owns semantics; the browser renders.** The projection supplies the event kinds,
correlations, cardinalities, identities, payload facets, and aggregates a rendering layer needs
to display and filter. A rendering layer does not parse raw NDJSON, redefine lifecycle semantics,
or invent a correlation the projection did not license.

**No persistence is required and no public schema is promised.** The projection is recomputed
from raw on demand. Nothing is cached, indexed, or written to disk. The types are serializable so
task:9 can hand them to a page, and that representation is internal and unstable: nothing outside
this repository may depend on its shape, and it is not an interchange format.

### On serving a projection over local HTTP

decision:5 deferred this and asked that it be decided explicitly rather than arrived at by
convenience. Decided, narrowly:

- **Yes:** task:9 may serve exactly one immutable projected snapshot through the foreground,
  capability-protected loopback process sprint:2 plans. One explicitly supplied recording, read
  once, projected once, served to the machine it is on.
- **The browser receives Rust's projection, not raw NDJSON.**
- **No** persistence, file watching, remote binding, or process lifetime beyond the invocation.
  A presentation layer must not smuggle in a daemon, and this permission does not become one.

This is an architectural constraint accepted ahead of its implementation, which is allowed. It is
**not** a test result, which is not. task:8 implemented no HTTP, no sockets, no capability
handling, no headers, and no browser launching, and exercised none of it. Every security property
of that transport is unimplemented and unverified until task:9 does it and says so.

## Consequences

- **The projection is where the epistemic invariant is now most likely to be violated**, because
  a violation there looks like a convenience rather than a lie. Receipts and examined scope are
  the testable form of that risk, and 35 focused tests hold it.
- **Renderings get cheaper and more constrained at the same time.** A page has no reason to parse
  a record, and no license to; whatever it wants to show has to exist in the projection first,
  with receipts, or it does not get shown.
- **The types are more awkward than a flattened model would be.** A schema-tagged correlation id
  cannot be compared to a bare string, and a count cannot be read without its scope. That
  awkwardness is the constraint doing its job at compile time.
- **Adding a field to a rendering may mean adding it to the projection first,** which is a real
  cost and the intended one. The alternative is a renderer that knows something Rust does not.
- **The projection is validated against synthetic recordings and one real session's measured
  shape.** It has never met a failure, a denial, an interruption, or a resume in real data,
  because no such recording exists. Its handling of those paths is tested and unexercised, which
  is exactly the distinction task:4 insisted on and is repeated here about this layer.
- **Borrowing rather than owning means a projection cannot outlive its replay.** That is a mild
  ergonomic constraint and an accurate one: the derived view is not a thing you keep after
  throwing the evidence away.

### Deliberately deferred

- **Whether the projection is ever persisted.** Still open, still satisfying decision:5's first
  condition either way, and now with an implementation in front of it that recomputes cheaply
  enough that nothing forces the question.
- **A stable public projection schema.** Not until something outside this repository needs one,
  and not without a version discipline of its own.
- **Rendering of recordings large enough to matter.** Replay reads the whole file into memory and
  the projection holds a ledger entry per record. Scale is settled by the specimen, not by a
  hypothetical.
- **Any correlation beyond `tool_use_id`, `tool_call_id`, and delivered agent identifiers.**
  dragon:3's question — what a delivered identifier *means* — is unanswered for `prompt_id` and
  only partly answered for `agent_id`. No further identifier gets a meaning here.
- **Every security property of local HTTP.** Named above, deliberately unclaimed, and task:9's
  to establish.
