---
id: ide_01KZ1SSY9HA8X0649QYFTR8KND
sequence: 2
kind: idea
status: parked
created: 2026-08-02
---

# Scarp: expose a collection's section template before the first artifact exists

## Problem

`scarp new --body-file <PATH>` fills an artifact "from a UTF-8 Markdown file whose `## `
headings name this collection's own sections" — but nothing exposes what those sections
are. `scarp new --help` names the flag without naming the headings, and `scarp list` and
`scarp show` only work on artifacts that already exist. For a collection you have not
created yet, the required headings are unknowable from the CLI.

The failure is quiet, which is the part that matters. A body file with a heading Scarp does
not recognize does not obviously explode; the writer finds out by creating the artifact and
reading it back. That makes the first use of each collection a guess, and it makes
`--body-file` — the flag that exists precisely so an agent does not hand-author front
matter — harder to use correctly than the thing it replaces.

## Sketch

Print the skeleton on request:

```sh
scarp new dragon --print-template
```

emitting the empty section scaffold to stdout, so it can be read, or redirected and filled
in:

```sh
scarp new dragon --print-template > body.md
```

A `scarp show --template <collection>` would serve equally well. The requirement is only
that the answer is available before the first artifact of a collection exists.

Making an unrecognized `## ` heading in a body file a hard error, naming the sections it
did expect, would close the same gap from the other direction and is arguably the more
valuable half.

## Boundaries

- Read-only. No files written, no repository state touched.
- Not a request for user-defined sections. Scarp owning the template is the feature; this
  is only about being able to see what it owns.
- Not a substitute for documentation, though it would keep documentation and behavior from
  drifting apart.

## Evidence

Observed during the WitnessGlass bootstrap (Scarp 0.2.0). Creating two decisions, two
dragons, a sprint, and four tasks by `--body-file` required knowing nine distinct section
headings across five collections in advance. The workaround was to `scarp init` a throwaway
repository in a scratch directory, create one artifact of every collection, read the
generated templates, delete the scratch repository, and only then write the real body
files. That produced correct artifacts on the first try in the real repository, but it
required creating a second Scarp repository purely to interrogate the tool.
