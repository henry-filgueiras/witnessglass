---
id: tsk_01KZ23SWSFZQQ13F986XXKXBSP
sequence: 5
kind: task
status: closed
sprint: spr_01KZ1SQTZ730K3VJMH127NMXNS
created: 2026-08-02
closed: 2026-08-02
---

# Harden raw-stream boundaries before first contact

## Objective

Bring the raw-stream implementation into conformance with decision:3 on three boundaries
that the emit-and-replay slice got wrong or left unguarded, before an adapter starts
producing recordings whose damage nobody controls.

This is a conformance repair, not a new architectural decision. Raw schema v1 and the event
vocabulary are unchanged. decision:3 and task:3 stay exactly as written; they are the
historical record of what was decided and what was built, and this task is the record of
where the build did not meet the decision.

The three boundaries:

**A truncated tail must stay opaque.** decision:3 states that replay "returns the valid
prefix, reports the recording as incomplete ... and never parses or presents the fragment as
an event." The implementation validates the entire input as UTF-8 *before* locating the final
newline, so a fragment cut midway through a multibyte character fails the whole read. The
intact prefix — every complete record the recording ever held — is condemned by bytes that
were never claimed to be a record. That is precisely backwards: the fragment is the part
nothing is known about, so it must be the part that is never inspected.

This is not hypothetical. A recording is cut short exactly when an emitter dies mid-write,
and a write interrupted inside a UTF-8 sequence produces invalid bytes by construction.

**Sequence exhaustion must be refused, not wrapped.** The appender computes the next sequence
as `record.sequence + 1` with no overflow check. At `u64::MAX` that panics in debug and wraps
to 0 in release. A wrapped sequence is the one thing the ordering contract cannot survive: it
silently restarts the canonical chain.

**Documentation must not claim more than the code does.** The README describes storage as
"verbatim" when object-key order, whitespace, escaping, and numeric rendering are normalized;
still calls the format "likely JSONL/NDJSON" after decision:3 settled it; and shows a worked
example whose displayed record does not match the invocation above it.

## Acceptance criteria

- `replay_bytes` splits its input as bytes at the final `\n` before any UTF-8 validation.
  Everything after that newline is an opaque fragment, never decoded and never parsed.
- Only the newline-terminated prefix is validated as UTF-8. Invalid UTF-8 inside a complete
  record remains corruption, reported with its line number.
- A recording with no newline at all yields zero records and a truncated tail covering the
  entire byte length, even when those bytes are not valid UTF-8.
- The valid prefix is available in every truncated-tail case, whatever the fragment contains.
- Tests cover: a valid record followed by an unterminated invalid-UTF-8 fragment; a recording
  consisting solely of invalid unterminated bytes; a record torn midway through a multibyte
  character; and invalid UTF-8 inside a newline-terminated record still being corruption.
- The appender uses checked arithmetic for the next sequence. A final record at `u64::MAX`
  produces a typed, clearly worded error, and the recording is left byte-for-byte unchanged.
- A focused test covers sequence exhaustion.
- `README.md`: the synthetic invocation and the displayed record are mutually consistent;
  "likely JSONL/NDJSON" is replaced by the decided NDJSON format; and the "verbatim" claim is
  replaced by precise wording — JSON values survive semantically, while whitespace, escaping,
  numeric rendering, and object-key order may be normalized before the bytes become recorded
  evidence.
- Raw schema v1 and the event vocabulary are otherwise unchanged. decision:3 and task:3 are
  not rewritten.
- `scripts/check.sh` passes and the repair is committed.

## Result

All three boundaries repaired. Raw schema v1 and the event vocabulary are unchanged, and
decision:3 and task:3 were left exactly as written.

### Truncated tails are now opaque

`replay_bytes` splits its input as bytes at the final `\n` before anything is decoded.
Everything after that newline is an opaque fragment: never decoded, never parsed, counted
only as a byte length and an offset. Only the newline-terminated prefix is validated as
UTF-8.

The old order was backwards in a way that mattered. Validating the whole input first meant
the one part nothing is known about — the fragment — could condemn every complete record in
front of it. And this was not an edge case: a recording is truncated exactly when an emitter
dies mid-write, and a write interrupted inside a multibyte character produces invalid UTF-8
by construction. The failure mode was that the most common form of truncation destroyed
access to the entire recording.

Invalid UTF-8 *inside* a newline-terminated record is still corruption, reported with its
line number. That record was written whole and is wrong, which is a different claim.

### Sequence exhaustion is refused

`record.sequence + 1` became `checked_add(1)`, with a new typed `Error::SequenceExhausted
{ last }`. Previously this panicked in debug and wrapped to 0 in release — and a wrapped
sequence is the single failure the ordering contract cannot survive, because it silently
restarts the canonical chain rather than failing. The refusal happens before the file is
opened for writing, so the recording is left byte-for-byte unchanged.

### Documentation drift repaired

In `README.md`:

- The worked example and the displayed record now agree. The example was missing its
  `observed_tool_started` step, so the record shown as `sequence: 3` did not correspond to
  anything the reader would have produced. The example now emits four records and the text
  says which one is displayed. Verified by running the example's exact payloads through the
  built binary and diffing the third replayed record against the README block.
- "likely JSONL/NDJSON" is replaced with the decided format: UTF-8 NDJSON, one complete
  record per newline-terminated line, one file per session.
- The "stores whatever the emitter hands it, verbatim" claim is replaced with precise
  wording. JSON values survive semantically — a string keeps its characters, a number its
  value, an object its keys — but whitespace, escaping, numeric rendering, and object-key
  order may be normalized before the bytes become recorded evidence. The privacy point is
  strengthened rather than softened: nothing is dropped and nothing is scrubbed, so a
  credential handed to the recorder is a credential in the recording.

The same inaccurate "verbatim" wording appeared on the `arguments` and `result` doc comments
in `src/record.rs` and was corrected there too — the same false claim, in another location.

### Files

- `src/replay.rs` — byte-first split; module docs state why the fragment stays opaque.
- `src/append.rs` — checked sequence arithmetic.
- `src/error.rs` — `SequenceExhausted` variant and its message.
- `src/record.rs` — corrected `arguments` / `result` doc comments.
- `tests/damage.rs` — five new tests, one renamed for accuracy.
- `README.md` — the three drift repairs above.

No dependencies added. `publish = false` retained.

### Tests

35 passing, up from 30. `damage.rs` went 13 → 18:

- `an_unterminated_invalid_utf8_fragment_does_not_condemn_the_prefix` — two complete records
  followed by an undecodable fragment; both records still replay.
- `a_recording_of_only_invalid_unterminated_bytes_replays_as_no_events` — no newline
  anywhere and nothing decodable; zero records and a truncated tail covering the whole file.
- `a_record_torn_midway_through_a_multibyte_character_keeps_its_prefix` — cut two bytes into
  a four-byte character, with an assertion that the cut genuinely tears the character so the
  test cannot silently stop testing what it claims to.
- `invalid_utf8_in_a_later_complete_record_is_still_corruption` — the boundary in the other
  direction, at line 2.
- `a_recording_at_the_maximum_sequence_refuses_further_appends` — typed error plus a
  byte-for-byte comparison of the file before and after.
- `invalid_utf8_is_corruption` renamed to
  `invalid_utf8_inside_a_complete_record_is_corruption`, since the old name now describes a
  case that is deliberately *not* corruption.

Full gate after the repair: `cargo fmt --check` clean, `cargo clippy -D warnings` clean,
35 tests passed / 0 failed, `scarp doctor` 13 artifacts with no problems.

### Manual verification

Against throwaway recordings outside the repository, using the built binary: an undecodable
unterminated fragment appended to a four-record recording still replayed all four with exit
2 and a 46-byte fragment reported at byte 1186; appending onto it was refused; a recording
whose final record carried `18446744073709551615` refused the append by name and its SHA-256
was identical before and after.

### Deviations and limitations

- **`CLAUDE.md` has the same stale format wording and was left alone.** Section 3 still says
  "likely JSONL/NDJSON". The repair was scoped to `README.md`, so this is reported rather
  than silently edited — but the durable agent contract now trails decision:3, and that is
  worth a one-line fix in some later pass.
- **A recording at `u64::MAX` is refused on append but is not replayable anyway**, since
  replay requires the chain to start at 1. The append path reads only the final record,
  which is why the guard is needed there independently.
- **Sequence exhaustion is unreachable in practice.** At one record per nanosecond it would
  take roughly 585 years. It is guarded because the failure is silent, not because it is
  likely.
- Truncation semantics are otherwise unchanged, and no behavior outside these three
  boundaries was touched.

### Scarp desire paths

**idea:1 recurred for the fourth time** — this result was written to a temporary file and
appended with a shell redirect before `scarp close task:5`. Four for four across every task
this project has closed. Nothing to add to the idea; the sample is simply now unambiguous.

**The artifact-linking friction noted in task:3 recurred, so it now meets the promotion
bar.** This task exists purely as a conformance repair to decision:3, and there is no way to
express that relationship in Scarp 0.2.0 — no `depends-on`, `refines`, or `conforms-to` edge,
and `close --resolved-by` only applies to dragons. The link lives in prose in the objective,
which `scarp resolve` can follow but nothing validates: if decision:3 were ever renumbered or
superseded, this task would quietly point at nothing. Second occurrence in two consecutive
tasks, so it is promoted to idea:3 per the desire-path rules rather than noted again.
