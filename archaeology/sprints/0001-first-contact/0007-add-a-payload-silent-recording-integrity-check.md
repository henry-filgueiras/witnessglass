---
id: tsk_01KZ28W5ZQDJWGXMZGG362EWYG
sequence: 7
kind: task
status: closed
sprint: spr_01KZ1SQTZ730K3VJMH127NMXNS
created: 2026-08-02
closed: 2026-08-02
---

# Add a payload-silent recording integrity check

## Objective

Give an operator a way to answer one question about a recording — is it complete, truncated,
or corrupt — without putting the recording's event payloads on the terminal.

This is tooling needed immediately after first contact, not a hook exercise. A recording made
from a real session contains prompts, commands, absolute paths, file contents, and tool
output, and the only tool that currently reads one is `witnessglass replay`, which prints
every record to stdout. So the first thing an operator wants to do with a fresh recording —
confirm it is intact — currently requires displaying material that dragon:2 says is unsafe to
display. Wanting to know whether the flight recorder survived the flight should not require
reading the flight.

The check is deliberately not a new parser. `replay` already distinguishes the three answers
and already encodes them in its exit status: 0 complete, 2 valid prefix with a truncated
tail, 1 corrupt or unreadable. A second implementation of that judgement would be a second
opinion about what a recording says, and the project has room for exactly one. The script
therefore adds no analysis at all; it discards replay's stdout, preserves its exit status,
and leaves its already payload-free summary on stderr.

No Rust subcommand is added here, and `src/` is deliberately untouched: the binary armed at
the start of the session that this task is being done inside must stay the sensor for the
whole of that session.

## Acceptance criteria

- `scripts/check-recording.sh <RECORDING>` exists, uses strict Bash error handling, resolves
  the repository root relative to its own location, and works on the macOS and Linux hosts
  this project claims.
- The already-built `target/debug/witnessglass` binary is the only thing invoked, and
  `replay --recording` is the sole parser and validator. No second implementation of
  completeness, truncation, or corruption.
- Replay's NDJSON stdout is discarded in full, and checking a recording puts nothing of the
  script's own on stdout either. `--help` prints usage there, as every other script here does.
- Exit status is preserved: 0 for a complete valid recording, 2 for a valid prefix with a
  truncated tail, 1 for corruption, an unreadable or missing recording, an invalid
  invocation, or a missing binary. The contract is exactly those three values.
- Replay's payload-free summary, or its error, stays visible on stderr.
- The recording is never altered, and the script never arms, disarms, builds, or looks inside
  the active recordings directory.
- Exactly one recording path is accepted, plus a conventional help option. A path containing
  spaces works.
- Payload silence is bounded and stated. Event bodies never reach stdout, which is discarded
  whole. Ordinary parser diagnostics — line numbers, byte offsets, schema versions, sequence
  numbers, session ids — are not payloads and are not hidden. The one exception is a *corrupt*
  record, whose diagnostic can quote the bytes the parser rejected; that limit is documented
  wherever the guarantee is stated, and is pinned by a test rather than left to prose.
- Automated coverage for at least: a complete recording exiting 0; a truncated tail exiting
  2; a corrupt newline-terminated record exiting 1; a missing recording exiting 1; a missing
  built binary exiting 1; no arguments and too many arguments; a recording path containing
  spaces; stdout empty in every case; and a distinctive synthetic payload marker appearing in
  neither stdout nor stderr.
- Every test uses synthetic recordings in temporary directories. No real recording is read,
  and no dependency is added.
- The README and the Claude adapter document introduce the check as the payload-silent way to
  validate a fresh recording's structure, and keep the standing warning that recordings are
  not redacted and are not safe to share.
- `scripts/check.sh` passes. task:4 stays pending, both dragons stay open, and `src/`, the
  Cargo manifests, and the hook configuration are unchanged.

## Result

`scripts/check-recording.sh <RECORDING>` exists and answers the one question an operator has
about a fresh recording — complete, truncated, or corrupt — without putting the recording on
screen. It is 90 lines, most of them explaining themselves, and it contains no parser.

The whole design is the delegation. `replay` already distinguishes the three answers and
already encodes them in its exit status, so the script discards replay's NDJSON stdout with
`>/dev/null`, passes its exit status through, and leaves its one-line summary on stderr. No
pipeline is involved, so `pipefail` cannot rewrite the verdict on the way back; the invocation
is `"$BINARY" replay --recording "$RECORDING" >/dev/null || STATUS=$?`, which is also what
keeps `set -e` from turning the expected exit 2 into a script failure. Every expansion is
quoted, so a path containing spaces is one `argv` element and never a glob.

Two places where the script does something other than pure delegation, both deliberate:

- **A missing or non-executable `target/debug/witnessglass` is caught before replay is
  attempted**, with a message naming `cargo build`. A missing build is an operator problem
  with a fix, not a statement about the recording, and letting it arrive as "command not
  found" would have filed it as one.
- **A status outside {0, 1, 2} is normalised to 1 and announced on stderr.** `replay` exits
  0, 1, or 2 and nothing else, so anything else means it never reached a verdict — an OOM
  kill on a large recording, a Ctrl-C — which is neither evidence that the recording is fine
  nor evidence that it is truncated. Exit 1 therefore means "this did not check out", not
  specifically "corrupt", and the help text and the adapter document now say so.

`src/` is untouched and no Rust subcommand was added, deliberately: the binary armed at the
start of this session had to remain the sensor for the whole of it. The sensor's SHA-256 was
still byte-for-byte the one `.witnessglass/armed` recorded at arming time when this work
finished, so running the gate did not relink it underneath the running session.

### The limit that had to be documented rather than fixed

Payload silence is absolute on stdout, which is discarded whole. On stderr it is bounded, and
the boundary is not where the first draft of this script claimed it was.

`Error::Corruption` carries `serde_json::Error::to_string()` verbatim, and serde's
`invalid type`, `invalid value`, `unknown field`, and `unknown variant` diagnostics all quote
the offending value or key **in full and untruncated**. A record whose `sequence` slot holds a
string produces `invalid type: string "…"` with the whole string in it; since v2 payload
fields carry arbitrary `serde_json::Value` tool input and responses, a corrupted record can
therefore put a multi-kilobyte tool output on stderr. That is precisely the material this
script exists to keep off a terminal, and it fires in the one case an operator most wants to
investigate.

It is real, it was reproduced end to end through the script, and it is **not fixed here**. The
fix belongs in `src/record.rs`, which this task is not allowed to touch, and the alternative —
filtering replay's stderr in the script — would make the check a second opinion about what a
recording says, which is the one thing it must not become. So the claim was narrowed instead:
the script header, the README, and `docs/claude-adapter.md` all now state that a *corrupt*
record's diagnostic can quote the bytes the parser rejected, and that a recording which checks
as corrupt is the one not to check on a shared terminal.

A prose caveat rots, so the limit is pinned by
`a_corrupt_record_can_quote_its_own_bytes_in_a_parser_diagnostic`, which asserts the leak
rather than its absence. If it ever fails, the limit moved: either it was fixed at the parser,
in which case the test should be deleted and the documents corrected, or it grew, in which
case the payload-silence claim is no longer accurate. That is the honest shape for a defect
that is known, bounded, and deliberately not fixed today.

### The review

One read-only subagent reviewed the script contract and the proposed test matrix. It started
and completed. It found the stderr leak above independently and demonstrated it with two
concrete shapes; it identified the corrupt-case test as false comfort, because the natural
fixture for "unparsable record" is a syntax error, which quotes nothing — so the test would
have certified a payload silence that does not exist; it found that the whole matrix asserted
absence and nothing asserted the verdict was still *visible*, meaning a script that silenced
replay entirely would have passed; it found that `--help` was matched before the arity check,
so `-h --extra` exited 0, the code that means "complete recording"; and it noted the
no-verdict branch was untested and reachable.

All five were fixed. The arity check now runs before the help match, so no malformed
invocation can reach exit 0. The matrix gained the verdict-visibility assertions, the
no-verdict branch, a directory in place of a recording, an empty recording, a
present-but-non-executable binary, an empty-string argument, and a whole-tree snapshot instead
of a recording-only one. Its remaining notes were nitpicks and are recorded as such: a file
named literally `-h` is read as the option (documented, workaround `./-h`), and the help-path
SIGPIPE it looked for does not fire because the usage text is far below a pipe buffer.

One of its claims was wrong in a useful way: it said the script's header clause about not
hiding "the caller-supplied filename" described something that never happens, since replay's
I/O errors never include the path. Correct, and the clause was rewritten rather than the
behaviour changed — the script printing its own path line was rejected as scope creep.

### Tests

`tests/check_recording.rs`, 18 tests, suite total **100** (was 82). Synthetic recordings only,
built from the existing `common::raw_record` / `common::ndjson` helpers, written into a
throwaway tree shaped like the repository in the manner `tests/arm_disarm.rs` already
established. No dependency was added. The repository's own recordings are never read by the
suite.

Covered: a complete recording exiting 0 with the verdict on stderr; a truncated tail exiting 2
with `INCOMPLETE` and the intact prefix still counted; a corrupt newline-terminated record
exiting 1; an empty recording exiting 0, pinned because it is a judgement rather than an
accident; a missing recording, a directory in place of one, a missing binary, and a
non-executable binary each exiting 1, the last two naming `cargo build`; a stubbed replay
exiting 3 normalised to 1 with `without reaching a verdict` on stderr; zero arguments, two
arguments, `--help <path>`, and `-h --extra` all refused; an empty-string path; a path
containing spaces; stdout empty across every one of those; a synthetic payload marker
reaching neither stream for the complete, truncated, and corrupt cases — including a marker
placed *inside* the unterminated fragment, which replay must never decode; the documented
stderr leak asserted rather than hoped for; and the whole fake-repository tree byte-for-byte
unchanged after checking, which also says the script wrote no scratch file, created no state
directory, and built nothing.

### Manual verification

Exercised by hand against synthetic recordings in a scratch directory outside the repository,
generated by driving `claude-hook` with synthetic payloads: complete (0), truncated (2),
corrupt (1), missing (1), no arguments (1), two arguments (1), `-h --extra` (1), `--help` (0),
and a path containing spaces (0). Two deliberate leak probes confirmed the stderr exception in
both of its shapes — a marker as an unknown key, and a marker as a string in a numeric slot.
No real recording was read at any point.

### Scarp desire paths

**idea:1 recurred for the sixth time.** This result was written to a temporary file and
appended with a shell redirect before `scarp close task:7`. Six for six. Nothing new to add.

**idea:2 recurred in the narrowed form task:6 predicted.** `scarp new task --body-file` needed
the collection's `## ` headings again, and reading an existing task file answered it in
seconds — but the file also revealed that `## Result` is *not* emitted by `--body-file` when
the body omits it, so the heading itself had to be appended by hand along with the content.
That is the same first-use gap, one layer down, and it is consistent with idea:2 being most
valuable to a new Scarp repository and progressively less so to an established one.

No new idea is filed. Nothing else in this session produced friction worth recording, and
manufacturing one for a task that exists partly to be recorded would be the exact opposite of
what desire-path evidence is for.

### Subject-session witness statement

**This is Claude's self-report, not observed ground truth.** It is what one participant
believes it did, written without consulting the recording, and it is offered to task:4 as an
independent basis for comparison — including in the places where it turns out to be wrong.
Where it disagrees with the captured evidence, the disagreement is the finding.

- **Baseline commit:** `3095e2db74368c952a256fb76b16fc933ab94e86`, worktree clean, branch
  `main`, matching `origin/main`.
- **Files intentionally changed:** created `scripts/check-recording.sh`,
  `tests/check_recording.rs`, and this task's archaeology file; modified `README.md` and
  `docs/claude-adapter.md`. Nothing else was intended to change, and `src/`, both Cargo
  manifests, `.claude/`, task:4, and both dragons were not touched.
- **Checks run:** `./scripts/check.sh` once at the start of the session (passed, 82 tests),
  once after implementation (passed, 100 tests), and once after this task was closed.
  `cargo fmt --all` was run once after the first post-implementation gate failed on formatting
  only, and it rewrote two statements in `tests/check_recording.rs`.
- **Manual synthetic verification actually run:** as listed above, all against synthetic
  recordings in a scratch directory outside the repository. `.witnessglass/recordings/` was
  never opened, listed, replayed, or copied.
- **Subagent:** one read-only review subagent was requested, started, and completed, returning
  findings that materially changed both the script and the test matrix. No other subagent was
  spawned.
- **Tool failures, permission denials, interruptions:** none noticed. No tool call was
  reported to me as denied, and no permission prompt was reported as refused. Nothing was
  deliberately provoked for coverage.
- **Final commit message:** `scripts: add payload-silent recording check`.

No tool-call counts are estimated here, deliberately. Counting is exactly what the recording
is for, and a self-reported number would give the comparison a false anchor.

### Concerns that should shape the task:4 comparison

- **Denial and failure were not exercised.** No permission denial, tool failure, or
  interruption was provoked, so an absence of `tool_denied` and `tool_failed` records means
  "not exercised", not "no denials occurred". task:6 already flagged that `PermissionDenied`
  may only fire from the auto-mode classifier; this session does not test that either way.
- **One subagent ran, which is the useful measurement here.** Whether `SubagentStart` and
  `SubagentStop` appear at all, whether `parent_agent_id` arrives populated, and — the larger
  question — whether the subagent's own tool calls produced hook events with a distinguishable
  `context.agent_id`, or produced nothing at all, is the open empirical question from task:6.
  A recording that shows the subagent as a single opaque pair of boundary events would be a
  significant blind spot to write down.
- **Several tool calls were deliberately issued as parallel batches.** If parallel hooks are
  what task:6 predicts, this recording should contain records whose `sequence` is recorder
  acquisition order rather than causal order. That makes this session a real test of the
  ordering caveat rather than a hypothetical one.
- **`cargo fmt --all` rewrote a file that no tool event will show being written.** The
  recording will show a `Bash` command and its reported output; the file mutation it caused is
  invisible, because completed hooks expose Claude's tool-level input and response and not
  descendant process effects. That is a clean, concrete instance of a blind spot task:4 can
  demonstrate rather than assert.
- **Reported intent should be plentiful and duplicated.** Nearly every `Bash` call in this
  session carried a `description`, so a reader counting occurrences of any such string will
  find it twice — once in `requested_input`, once as a `reported_intent` record. That is by
  design and documented, but it will look like double-counting to anyone who has not read
  decision:4.
- **Hook latency was not perceptible and was not measured.** Nothing in this session times
  anything, so "it felt fine" is worth exactly what it sounds like.
- **The recording holds ordinary repository content plus absolute paths** under this host's
  home directory and a session scratch directory, and the full text of every file read. No
  credential was knowingly handled, which is not the same as none being present. dragon:2
  stands.
