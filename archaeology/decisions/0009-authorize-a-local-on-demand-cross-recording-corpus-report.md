---
id: dec_01KZEXT31BWP6PVST2BJ1ZHB2C
sequence: 9
kind: decision
status: accepted
created: 2026-08-07
---

# Authorize a local, on-demand cross-recording corpus report

## Context

Every projection this repository has built reads **one** recording. decision:5 lifted the
user-interface non-goal and sprint:2 spent that lift on `witnessglass view --recording <PATH>`, a
foreground viewer over one explicitly supplied file. The motif experiments do compare two recordings —
`cross_pairs` has ranked windows across a pair since sprint:9 — but they are experiments, they run over
decision:8's four admitted specimens, and their output is an evidence page about a detector rather
than an account of a corpus.

A human operator now has a directory of recordings from a second project and wants to know what
recurs across them. That is a question no existing surface answers, and answering it needs three
things this repository has never sanctioned in one place: reading a *directory* rather than a file,
holding several recordings in memory at once, and emitting a **written report** derived from all of
them.

Two standing non-goals are adjacent and must not be quietly widened. §6 forbids "a daemon, background
service, or anything that outlives the command that started it" and forbids "hosted access, remote
binding, upload, or export of a recording in any form". A directory-walking analyser is one
`--watch` flag away from the first and one `--publish` flag away from the second, and the distance
between "reads a folder" and "indexes a folder" is a design decision, not a fact of nature.

A third pressure is epistemic. sprint:19 and sprint:20 spent two rounds establishing what the complete
search can and cannot claim, and sprint:20's answer was a collapse: on the raw event projection, the
separation the search measured is fully explained by first-order categorical transition structure. A
new, more legible projection — one that groups a request with its outcome and labels the pair with a
category — is exactly the kind of surface on which that hard-won result gets silently reused for a
projection it was never measured on.

## Decision

**A local corpus report is authorized, once, in the shape below, on human authorization recorded in
sprint:21.**

### What is authorized

- **One command, run on demand, over a directory the operator names.** It reads, analyses, writes its
  outputs, and exits. It is the same shape as `view`: foreground, explicitly invoked, dead when the
  command that started it is.
- **Cross-recording analysis.** Several recordings held in memory at once, replayed and validated
  through `replay_file` and `inspection` — never through a second reader of raw NDJSON, per decision:6.
- **Derived, disposable aggregate output**: a machine-readable `facts.json`, a `manifest.json`, and a
  `report.md` rendered *from* those facts. All three are projections in the sense of §3: rebuildable
  from the recordings, never rewriting them, safe to delete.
- **Deterministic, rule-based reporting.** Fixed seeds, stable ordering, no clock inside the
  deterministic outputs, and no language model anywhere in the pipeline.
- **Comparison of two independently produced `facts.json` documents.**
- **A second derived projection — the observed tool-action stream — and a versioned category
  vocabulary over it,** because the raw event projection is too close to recorder grammar to be the
  only human-facing lens.

### What is not authorized, and stays not authorized

- No daemon, watcher, index, cache, or state that outlives the command.
- No network, no upload, no export, no share affordance, no hosted mode.
- No recording copied into tracked storage, and no generated report committed.
- No language model, local or remote, inside the analysis.
- No new product subcommand. The surface is `cargo run --example corpus-report`, which keeps the
  product binary free of any dependency on `crate::experiment` and keeps the whole workflow deletable
  in one commit.
- No claim that a report is redacted, sanitized, or safe to share. §5 stands unchanged.

### The category vocabulary, and what it is not

The workflow projection labels each observed action with a category drawn from a small versioned
vocabulary. **A category is analyser shorthand for a delivered tool name, and nothing else.** It is not
the agent's intent, not a reported claim, and not an observed fact about what a command did. Reported
intent is never merged into an observed action; a `reported_intent` record contributes no action.

Shell commands are classified by their leading program name only. Where that requires looking at a
command string, **only the resulting category and the record's sequence number are retained** — never
the command, never a fragment of it, never a token of it. A command whose leading program is not in the
vocabulary is `Shell`, and anything else unmapped is `Other`. There is no imaginative classification and
no fallback that guesses.

### What a report may contain

Exactly what decision:8 permits, and no more: opaque session prefixes, record and event counts,
delivered tool names, derived categories, numerical quantities, and raw sequence-number receipts. No
prompt text, response text, reported-intent text, command text, tool output, file contents, paths,
or host and user identity.

### What a report may claim

- A **descriptive** result — that a shape recurs across N of M eligible sessions — needs no null and is
  reported as description.
- A **calibrated** result needs a null measured **on the projection it is claimed for**. sprint:20's
  collapse was measured on the raw event projection under the exact first-order doublet null, and it
  may not be transferred to the workflow projection by assertion. The workflow projection carries its
  own calibration or its output is labelled descriptive.
- The corpus statistic a report calibrates — cross-session prevalence of an exact recurring shape — is
  **not** sprint:19's `T`, and no round's verdict about `T` may be cited for it.

## Consequences

- The one-item precedent of §6 is not widened. decision:5's lift concerned a *user interface*; this
  decision concerns a *cross-recording analysis command*, and it is authorized on its own evidence
  rather than on decision:5's. Every other item on the non-goal list stands at full strength.
- The recordings this workflow is run against are **not** admitted as specimens. decision:8's inventory
  is unchanged by this decision. A corpus report over an untracked directory is a local exploratory
  artefact; admitting any of its recordings as a repository-cited specimen would amend decision:8 in
  its own round, with the counts that decision permits and nothing else.
- Generated real-corpus output lives outside the repository or under an ignored path, and is treated as
  exactly as sensitive as the recordings behind it.
- The workflow is `src/experiment/corpus.rs` plus `examples/corpus-report.rs` plus
  `tests/corpus.rs`. Deleting those three files and their fixtures deletes the capability, and nothing
  in the product depends on any of them.
