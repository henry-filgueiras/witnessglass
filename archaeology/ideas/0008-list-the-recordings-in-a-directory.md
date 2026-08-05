---
id: ide_01KZA4FTN5Y22V34B5K1Q176K5
sequence: 8
kind: idea
status: parked
created: 2026-08-05
---

# List the recordings in a directory

## Problem

Nothing tells an operator which recordings exist.

`view` refuses to pick one, deliberately and correctly: "a viewer that guesses which recording
you meant is a viewer that can open the wrong one." That refusal is not the problem and should
not be relaxed. The problem is that no other affordance takes its place, so finding the argument
to hand `view` falls back to

```sh
ls -t .witnessglass/recordings/*.ndjson | head -1
```

Recordings are named by session UUID, which carries no time, no size, and no indication of what
happened in them. Choosing between several therefore means stat'ing files and guessing from
mtime.

Inside this repository the friction is small, because the recordings are few and the operator
wrote the tooling. An external project feels it immediately: it has no `scripts/`, and the
discovery procedure has to be written down somewhere in that project instead — which is exactly
the WitnessGlass knowledge an observed project should not need to carry.

## Sketch

A read-only listing over a directory:

```sh
witnessglass recordings --dir <DIR>
```

One line per recording: session id, the span between the first and last `recorded_at`, size,
record count, and the same verdict `check` would give (see idea:6). Enough to identify the one
you meant before handing it to `view`.

## Boundaries

- **`view` keeps refusing to guess.** No `--latest` flag on `view`, and no implicit hand-off:
  this lists, the operator chooses. A listing that `view` consumed automatically would
  reintroduce exactly the failure the refusal prevents.
- Payload-silent. Session ids, counts, sizes, and timestamps are not event payloads; nothing
  from inside a record may reach this output.
- Counting records means replaying them. The cost, and the fact that a corrupt recording's
  diagnostic can quote bytes, both need stating rather than discovering.
- Not an index, not a database, not cross-session comparison — all three are standing non-goals.
- No recursive scan and no default directory that guesses where recordings live.

## Evidence

Encountered on 2026-08-05 while commissioning cuecraft. The discovery procedure had to be
invented and then documented outside this project, in cuecraft's own archaeology and in the
commissioning report, because there was nothing to point at. See log:1.
