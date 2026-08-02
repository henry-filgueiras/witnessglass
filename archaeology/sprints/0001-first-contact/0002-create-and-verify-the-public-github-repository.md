---
id: tsk_01KZ1SR0ZRQ9FMW7FGXPG81HZK
sequence: 2
kind: task
status: pending
sprint: spr_01KZ1SQTZ730K3VJMH127NMXNS
created: 2026-08-02
---

# Create and verify the public GitHub repository

## Objective

Create `henry-filgueiras/witnessglass` as a public GitHub repository, configure its
description and topics, push the bootstrap commits to `main`, and verify that the remote,
visibility, default branch, pushed commit, and inaugural CI run are all as intended.

Repository creation is a one-way public act, so availability is reconfirmed immediately
before creation rather than relying on the earlier preflight.

## Acceptance criteria

- The name is reconfirmed unclaimed immediately before creation; an authentication,
  network, or authorization failure is treated as a blocker rather than as evidence of
  availability.
- The repository exists at `https://github.com/henry-filgueiras/witnessglass`, is public,
  and has the positioning line as its description.
- Topics are set to `rust`, `ai-agents`, `observability`, `developer-tools`, and
  `flight-recorder`.
- `origin` points at that repository and local `main` tracks `origin/main`.
- The bootstrap commit is present on the remote `main` and the pushed SHA is recorded.
- The inaugural GitHub Actions run is inspected. If it fails, only bootstrap-related
  failures are diagnosed and repaired: fixed locally, re-gated through `scripts/check.sh`,
  committed, pushed, and verified again.
- CI concludes successfully and its run URL is recorded.
- No crate is published, no tag is created, and no release is cut. No branch protection,
  secrets, deployments, or external services are added.
- The archaeology recording this inauguration is itself committed and pushed, and the final
  worktree is clean.
