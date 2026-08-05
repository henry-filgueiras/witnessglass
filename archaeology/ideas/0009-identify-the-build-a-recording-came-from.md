---
id: ide_01KZA4FTNAAZRHRX9A06YA58K7
sequence: 9
kind: idea
status: parked
created: 2026-08-05
---

# Identify the build a recording came from

## Problem

`witnessglass --version` prints `witnessglass 0.1.0`, and that is the whole of what a build can
say about itself.

While the crate is unpublished, every install comes from a local checkout, so `0.1.0` names a
moving target. The binary commissioned into cuecraft on 2026-08-05 was built from `d7cf64b`; a
build from any other revision reports the same string, and nothing distinguishes them. "Which
code produced this?" is currently unanswerable from the binary.

`arm.sh` gets closest — its sentinel records `witnessglass_version`, `binary`, and
`binary_sha256` — but the sentinel stays in the checkout that armed, is not written by any other
path, and a hash identifies a build without saying what it was built from.

This bites hardest in exactly the situation the project is in: local builds installed into other
projects, where the recorder and the recording are the only artifacts, and neither can name the
code behind it.

## Sketch

Embed the revision at build time and print it:

```
witnessglass 0.1.0 (d7cf64b 2026-08-05)
```

A build script that resolves the git description, falling back cleanly when there is no
repository to ask — a tarball, a vendored build, `cargo install` from a registry once one
exists — since a build that cannot name its revision must still build.

## Boundaries

- No network, and no build-time failure when git is absent or the checkout is not a repository.
- Dirty working trees should be visibly dirty rather than silently reported as the commit they
  were nearly.
- **This idea is about the binary, not the recording.** Whether build identity belongs *inside*
  a recording is a schema change and a separate question, and it arrives entangled with repo
  identity and the privacy posture that drops `cwd` — see log:1. Adding a field to a record is
  the kind of change §7 and the adapter's own `DELIBERATELY_UNRECORDED` discipline say needs a
  decision, not a quiet addition.

## Evidence

Encountered on 2026-08-05 while commissioning cuecraft, during the provenance audit that round
called for. A recording made by the installed binary carries `adapter`, `mechanism`, and
`channel`, and nothing that identifies the recorder that wrote it. See log:1.
