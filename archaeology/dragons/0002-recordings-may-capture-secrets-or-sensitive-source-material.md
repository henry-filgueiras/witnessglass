---
id: drg_01KZ1SQTZ4867HDQFRFPT4ZRNY
sequence: 2
kind: dragon
status: open
created: 2026-08-02
---

# Recordings may capture secrets or sensitive source material

## Context

A recording that is faithful to an agent session is, by construction, a recording of
whatever that session touched. The surfaces that are likely to appear in a raw stream
include:

- prompts and agent narration, which routinely quote source, configuration, and error text
- file contents and diffs, including files never intended to leave the machine
- shell commands and their arguments, where tokens are frequently passed inline
- command output and stack traces, which leak connection strings, headers, and keys
- environment variable reads and dumps
- absolute filesystem paths, which carry usernames, client names, and directory structure
- URLs with embedded credentials or signed query parameters
- test output referencing fixture credentials that are real more often than anyone admits

The tension is structural rather than incidental. Fidelity is the product. Every mechanism
that removes sensitive material removes evidence, and every mechanism that preserves
evidence preserves whatever was sensitive in it. There is no configuration of this system
in which the two goals do not trade against each other.

What makes this a dragon rather than a task is the claim surface, not the filtering. Regex
scrubbing of common key shapes is easy to write and easy to demonstrate on a good day. It
is also unable to recognize a credential that does not look like one, and the moment a tool
advertises redaction, users reasonably start sharing recordings — attaching them to bug
reports, pasting them into issues, sending them to colleagues. An unreliable filter that is
described as reliable is more dangerous than no filter at all, because it converts a
recording users treated as sensitive into one they treat as safe.

The honest posture before that contract exists is that recordings are sensitive artifacts
that stay on the machine that produced them.

## Question

What capture and redaction contract can WitnessGlass actually honor, and what exactly may
be claimed about the safety of sharing a recording?

## Constraints

- Recordings are potentially sensitive by default, with no exceptions assumed.
- No silent or magical redaction. Anything removed, truncated, or transformed must be
  visible as such in the record, so a reader can tell that something was there.
- Real recordings must not be committed to this repository during bootstrap. Any committed
  fixture is synthetic and obviously so.
- No claim of redaction, sanitization, or safe sharing may be made until the contract is
  implemented and tested. This applies to the README, the CLI, and any future export path.
- Redaction is a transformation on the way out, not a rewrite of raw evidence in place —
  the canonical stream stays canonical.
- Whatever is decided must degrade honestly: an unrecognized secret that survives filtering
  is a known limit to state, not a bug to discover later.

## Candidate direction

Treat the safe artifact as an explicit derived export rather than a property of the raw
recording. Raw stays raw, local, and sensitive by default. Sharing goes through a separate
step that is allowed to be lossy, that marks every elision in place, and whose limits are
documented in the same breath as its capabilities. Default posture in the meantime: the
recording does not leave the machine, and the documentation says so plainly.

## Resolution criteria

This dragon is resolved when:

- The sensitive surfaces that capture can actually encounter are enumerated against real
  recorded sessions rather than imagined ones.
- A capture and redaction contract is written down, stating what is captured, what is
  transformed on export, what is not attempted, and what is known to slip through.
- That contract is implemented and covered by tests, including tests that assert redaction
  is visible in the output rather than silent.
- User-facing documentation makes a claim no stronger than what the tests demonstrate, with
  the residual risk stated in the same place.
- The default behavior for an unconfigured user is the conservative one, and the
  destructive or exporting path is the one requiring an explicit act.

## Findings from first contact (task:4)

This dragon's context enumerated sensitive surfaces that capture was *likely* to encounter.
One real recording now says which of them actually appeared, and in what proportion. The
recording was read but nothing from it — no line, path, command, or fragment of output — is
reproduced here or anywhere else in this repository.

**Volume, from one 17-minute session of ordinary repository work: 580 KB across 234 records.**
Of that, 58% is tool response bodies, 24% is tool input, and 0.3% is reported-intent text. The
median record is 652 bytes; the 90th percentile is 6.2 KB; the largest single record is 34 KB.
A day of agent work at this rate is tens of megabytes of unredacted material per session
directory.

### Which of the predicted surfaces actually appeared

Confirmed present, by direct measurement:

- **Absolute filesystem paths carrying the host's username** — in 56 of 234 records, roughly a
  quarter of the file. A session-scoped scratch directory path appears in 16 more.
- **Full file contents**, in both directions: every `Read` response and every `Write` input is
  the complete text.
- **Complete shell commands with all arguments**, 64 of them.
- **Complete command output**, including the full text of files printed with `cat`.
- **Agent narration**, as 65 `reported_intent` records — and, because decision:4 duplicates
  rather than moves the description, each of those strings is in the file **twice**.

Predicted and **not** present in this particular session: environment dumps, URLs with
embedded credentials, and connection strings. That is a property of what this session
happened to do, not of the adapter, and says nothing about the next one.

### The finding that most sharpens the dragon

**The recording contains at least one string deliberately shaped like a credential which is in
fact a synthetic test marker.** The subject session was writing tests for payload silence and
used a marker string built to look like a secret, precisely so a leak would be conspicuous.
That marker is now sitting in a real recording.

This is a concrete instance of the dragon's core argument, arrived at by accident rather than
by construction. A regex scrubber run over this recording would flag that string and would be
wrong; the same scrubber would have no way to recognize a real credential written in a shape
it does not know. **The recording contains both false positives and, potentially,
unrecognizable true negatives, and no filter can tell them apart from the bytes alone.** An
unreliable filter described as reliable is the failure mode this dragon exists to prevent, and
first contact produced the evidence for it in the first session recorded.

A second, quieter point in the same direction: **agent narration is in the file twice by
design.** Any future redaction contract has to handle the same sensitive string appearing in
two records with different channels and different provenance, and removing one occurrence
while leaving the other would be worse than removing neither, because it would look redacted.

### What did not go wrong

The privacy posture held mechanically. `.witnessglass/` was gitignored and untracked
throughout; the recording was never committed, excerpted, or moved; the archivist session ran
disarmed so it did not record itself; and `scripts/check-recording.sh` answered "did the
recorder survive" without putting any of the 580 KB on the terminal. The one documented
exception to payload silence — a corrupt record's parser diagnostic quoting the bytes it
rejected — did not fire, because the recording was not corrupt. It remains the case where the
payload-silent check is least useful, and it is still correctly documented as such.

### What this changes about the resolution criteria

The first criterion — enumerate sensitive surfaces against real recordings rather than
imagined ones — is now met for **one** session shape. It should not be treated as met
generally: a session that runs a deploy, reads a `.env`, or handles an API token would extend
this list, and none of that has been recorded.

Nothing else changes. No redaction is implemented, no export path exists, and no claim about
safe sharing may be made. The measured volume above makes the case that the export path, when
it exists, will need to be selective rather than exhaustive — 580 KB of unredacted material to
convey what a 17-minute session did is not an artifact anyone will read, and "attach the
recording" will be the tempting wrong answer.

## The "attachable session" experiment, and the two ways it did not hold

Appended 2026-08-03. `docs/hostile-recording.md` designed a session intended to be safe to attach
to an upstream bug report: the agent was confined to `/tmp/wg-probe`, forbidden the repository,
`git`, `$HOME`, and the environment, and every file it wrote was labelled synthetic. The runbook
was careful to say "should is not does" and to insist a human read the recording before attaching
it. That caution was warranted. Two things leaked anyway, and neither came from the agent
disobeying — it complied throughout.

**Scope: Claude Code 2.1.220, macOS arm64, one 40-record session, plus its 14 raw probe payloads.**

### The sandbox does not bound the payload

The runbook's rationale was that "the adapter does not capture `cwd`, prompts, or transcript
paths". That is true of the *fields* and false of the *content*.

- **An error string carried the working directory.** The deliberate `Read` failure returned
  `File does not exist. Note: your current working directory is /Users/henry/witnessglass.` The
  adapter drops the `cwd` field faithfully, and the integration put the same value inside `error`,
  where it is stored as delivered because storing an error as delivered is the whole point.
- **A tool input carried a prompt.** The `Agent` call's `tool_input` contains the full subagent
  prompt, which quoted the repository path three times — because the sandbox instruction itself
  named the directory the agent was forbidden to touch. The instruction that made the session
  safe is the thing that put the path in the recording.

Neither is a defect. Both are the adapter doing exactly what §2 requires: store what arrived,
uninterpreted. The lesson is about the *claim*, not the code — **confining what an agent may do
does not confine what its payloads contain**, because the payloads are written by the harness and
by tool authors, not by the agent. Any future "this session is attachable" argument has to be made
by reading the artifact, never by reasoning about the sandbox.

### The payload-quiet alternative is not the quiet one

The runbook advised that for a bug report about timing, "you probably do not want the recording at
all — you want the probe output, which is a handful of raw payloads rather than a few hundred KB."
Smaller, yes. Less exposing, no. **Every probe payload carries `cwd` and `transcript_path`
verbatim** — including a `$HOME`-rooted path to the session's own transcript — which are precisely
the two fields the adapter deliberately drops. On this axis the raw capture is a strict superset of
the recording it was installed to audit.

Size is not the privacy axis. 26 KB of raw payloads that name the operator's home directory on
every line is a worse thing to attach than a larger file that does not, and the runbook currently
recommends the worse one. That advice needs correcting; `scripts/probe.sh` already says a raw
payload is as sensitive as a recording, and the runbook contradicted it.

### What this changes about the resolution criteria

The first criterion — enumerate sensitive surfaces against real recordings — gains a second
session shape, and a category the first pass could not have found: **surfaces that arrive inside
payload values rather than as payload fields**. A field-level inventory of what an adapter captures
will systematically miss them, in the same way a scan of adapter output missed a dropped field in
dragon:1. Both errors have the same shape: auditing a structure instead of its contents.

Nothing else changes. No redaction is implemented, no export path exists, and no claim about safe
sharing may be made. This session was designed to be the exception and it is not one.
