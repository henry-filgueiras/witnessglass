---
id: ide_01KZA4FTMZA0AMJKKPD7ET764H
sequence: 7
kind: idea
status: parked
created: 2026-08-05
---

# Arm and disarm a repository that is not this one

## Problem

Arming is bound to this checkout, twice over.

`scripts/arm.sh` derives `ROOT` from its own location, so it can only arm the repository it
lives in. The committed hook configuration it installs,
`.claude/settings.witnessglass.example.json`, names the binary as
`${CLAUDE_PROJECT_DIR}/target/debug/witnessglass` — and `CLAUDE_PROJECT_DIR` is the *observed*
project's root. Inside this repository those two roots coincide and the path resolves. In any
other repository it resolves to a path that does not exist.

So instrumenting an external project means hand-authoring the hook configuration: eight hook
surfaces, one block each, with the recordings directory repeated in every one. Everything
`arm.sh` provides is lost in the process — the rebuild that stops a stale binary from quietly
recording with old code, the synthetic-payload self-test, the refusal to arm if the adapter
writes to stdout, the sentinel recording what arming did, the displace-and-restore of a
pre-existing settings file, clean re-arming, and a disarm that removes exactly what was
installed and nothing else.

None of that machinery is wrong. It is simply unavailable to the case it would help most, since
an external operator is the one least likely to know what a correct hook block looks like.

## Sketch

Make arming a verb over a target directory:

```sh
witnessglass arm [DIR]        # defaults to the current directory
witnessglass disarm [DIR]
```

with the same gates `arm.sh` already applies, and hooks emitted in Claude's exec form naming the
command as bare `witnessglass` rather than a path, so an installed binary is found on `PATH` and
the written configuration contains no machine-specific location. `scripts/arm.sh` becomes a
wrapper that arms this repository, which is what it already is.

## Boundaries

- **Never take ownership of a settings file it did not write.** `arm.sh`'s displace-and-restore
  discipline is the floor, not a bonus. An external repository's `.claude/settings.local.json`
  may hold configuration from several other tools, which is a harder case than this repository
  has ever faced, and merging into someone else's settings is a different and larger problem
  than writing a file.
- Does not settle where an external project's recordings belong. That question is open — see
  log:1 — and this verb should not answer it by accident.
- No global Claude configuration. Per-project only.
- Whether arming is the right shape at all is not assumed here: a launcher that configures one
  ephemeral process and leaves nothing behind is a different product with different failure
  modes, and log:1 records that fork rather than resolving it.

## Evidence

Encountered on 2026-08-05 while commissioning cuecraft. The hook configuration was written by
hand into `cuecraft/.claude/settings.local.json`, untracked, and none of `arm.sh`'s guarantees
applied to it; the adapter self-test was re-run by hand against the installed binary to recover
one of them.

One qualification, because it matters for the sketch above: that Claude's exec form resolves
`command` on `PATH` is read from the hooks reference (re-read 2026-08-05), **not measured**. It
has not yet been exercised by a live session — no Claude session has been recorded in cuecraft
at the time of writing. See log:1.
