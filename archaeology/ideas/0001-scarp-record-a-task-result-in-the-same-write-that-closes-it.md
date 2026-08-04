---
id: ide_01KZ1SSY9D03KFBH9D2NE1Q4MQ
sequence: 1
kind: idea
status: parked
created: 2026-08-02
---

# Scarp: record a task result in the same write that closes it

## Problem

A task's outcome has to be written into the task before it is closed. Scarp owns task
creation (`scarp new task --body-file`) and the lifecycle transition (`scarp close task:N`),
but there is no command that writes the result, and the task template has no section for
one. The only way to record what happened is to hand-edit the artifact file and append a
`## Result` heading — the one class of operation the tool otherwise deliberately owns, and
the one the agent contract in this repository explicitly tells agents not to do.

That leaves the highest-value part of the archaeology as the least supported. The result
section is what turns a task from a plan into evidence, and it is written by the mechanism
Scarp is designed to replace.

It also splits an atomic act into two: append the result, then close. If the close is
forgotten, or the append happens after the close, the corpus is briefly inconsistent in a
way nothing checks.

## Sketch

Let the outcome ride along with the transition:

```sh
scarp close task:1 --result-file result.md
```

The result is appended to the task and the status moves to closed in the same write. A
standalone `scarp result task:1 --body-file result.md` for the case where the outcome is
recorded before the task is ready to close would cover the rest.

Either way Scarp keeps ownership of the heading name, its placement, and the file layout,
which is the actual point.

## Boundaries

- Not a general "append arbitrary section" command. The result is a known part of a task's
  shape; a generic editor would give up the guarantee that makes Scarp worth using.
- Should not require the result to exist to close a task. Some tasks legitimately close
  with nothing to add.
- No opinion here on whether closed tasks stay editable afterwards.

## Evidence

Observed during the WitnessGlass bootstrap (Scarp 0.2.0), closing task:1 and task:2 of
sprint:1. Both required the same workaround: write the result body to a temporary file,
append it to the task with a shell redirect, then run `scarp close`. Two occurrences in the
first session of use, and the pattern will recur for every task the project ever closes.

### 2026-08-04: upstream shipped this, and what it did not ship

Scarp now writes a task's result in the same write that closes it. The
flag is spelled `--body-file` rather than the sketched `--result-file`,
reusing the name `scarp new` already uses for the same job:

```sh
scarp close task:1 --body-file result.md
```

Against this idea's Sketch and Boundaries, point by point:

- **The outcome rides the transition.** The `## Result` section, the
  status change, and the `closed:` stamp land in one atomic write, so
  the two-step inconsistency described above is gone.
- **Scarp keeps ownership of the heading**, its placement, and the
  layout. A body file that writes `## Result` itself is refused, naming
  the reason.
- **A task can still close with nothing to add.** Omitting the flag
  closes exactly as before and appends nothing.
- **Not a general append command**, as this idea's first boundary asks.
  Dragons and sprints get their own terminal sections (`Resolution`,
  `Retrospective`, both dated); nothing else can be appended.
- **The standalone form was not built.** `scarp result task:1
  --body-file ...`, for recording an outcome before the task is ready to
  close, does not exist. Nor does anything for adding narrative to an
  artifact that already exists — which is what idea 4 asks for, and it
  remains open.

Also shipped alongside it, unasked: `[[kind:N]]` references in prose
supplied to `--body-file` are resolved and rewritten to their canonical
stable-id form at write time, on both the creation and the close paths.

Disposition of this idea is left open deliberately. The mechanism it
asks for exists; whether the missing standalone form is worth keeping it
parked for is a call for this project, not for the tool.
