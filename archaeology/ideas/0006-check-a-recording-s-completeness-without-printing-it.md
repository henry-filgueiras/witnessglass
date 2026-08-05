---
id: ide_01KZA4FTMTGJ24YRZHM39DA61P
sequence: 6
kind: idea
status: parked
created: 2026-08-05
---

# Check a recording's completeness without printing it

## Problem

`scripts/check-recording.sh` answers the one question an operator has after a session — did the
flight recorder survive the flight — without printing the flight. The README recommends it for
exactly that reason: `replay` answers the same question by putting every prompt, command, and
file body on the terminal.

It is unreachable from an external project. The script resolves its binary as
`$ROOT/target/debug/witnessglass`, where `$ROOT` is its own checkout, and refuses with
"run 'cargo build'" otherwise. A project that installed WitnessGlass as a binary has no debug
build to point at, so the payload-silent check is the one capability the installed CLI cannot
reach.

The gap is narrow but badly placed: the affordance that exists specifically to avoid displaying
sensitive evidence is the one that pushes an external operator back to `replay`.

## Sketch

Promote it to a verb:

```sh
witnessglass check --recording <PATH>
```

Exit 0 complete, 2 truncated tail, 1 corrupt or unreadable — `replay`'s own codes, with its
one-line summary on stderr and nothing on stdout. The script already documents that `replay` is
the sole parser and validator and that it adds no analysis of its own, so this is a relocation,
not a new implementation. `scripts/check-recording.sh` then becomes a thin wrapper over the verb
or goes away.

## Boundaries

- No analysis beyond the verdict. The script's discipline — one implementation of what a
  recording says — is the reason it is worth moving rather than reimplementing.
- Does not make a recording safe to share, and must not be worded as if it did. Checking is not
  redacting.
- The documented exception stands unchanged: a *corrupt* record's parser diagnostic can quote
  the bytes it rejected, so a recording that checks as corrupt is still the one not to
  investigate on a shared terminal.
- Not a directory scanner and not a "check the latest one" affordance. One explicitly supplied
  recording, in keeping with how `view` takes its argument.

## Evidence

Encountered on 2026-08-05 while commissioning WitnessGlass into cuecraft, an external repository,
with a binary installed by `cargo install --path <checkout> --locked`. The workaround used was

```sh
witnessglass replay --recording "$REC" >/dev/null; echo $?
```

which is precisely what the script runs, minus its argument validation and its explicit handling
of a `replay` that exits without reaching a verdict. See log:1.
