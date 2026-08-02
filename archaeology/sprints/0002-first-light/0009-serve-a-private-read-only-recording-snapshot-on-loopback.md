---
id: tsk_01KZ2CEB6K6GZAQ0FJ6N4AGF16
sequence: 9
kind: task
status: pending
sprint: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
created: 2026-08-02
---

# Serve a private read-only recording snapshot on loopback

## Objective

Add `witnessglass view --recording <PATH>`: a foreground, short-lived process that validates and
projects one explicitly supplied recording, holds it as an immutable in-memory snapshot, and
serves it read-only to a browser on loopback behind an unguessable per-launch capability.

**Depends on task:8.** The projection it serves must exist and be tested first; this task adds a
boundary around it, not a second interpretation of what a recording says.

The security posture is the deliverable as much as the transport is. A recording contains
prompts, commands, absolute paths, full file contents, tool output, and any credential that
passed through any of them (dragon:2, measured in task:4). Handing that to a browser is the point
of the sprint and is also the largest new exposure this project has created, so loopback binding
is treated as one layer and not as the answer.

The process exits when the command exits. **It is not a daemon**, does not watch the source file,
and gains no background mode, per decision:5.

## Acceptance criteria

- `witnessglass view --recording <PATH>` and `--no-open` exist, with `--no-open` the path tests
  and remote terminals use.
- The recording is validated and projected **before** a browser is opened. A corrupt recording
  fails before anything renders; a truncated one is served with its truncation carried through.
- Exactly one immutable in-memory snapshot is loaded at startup. Nothing re-reads, watches,
  tails, or refreshes the source file.
- The listener binds only to an OS-selected port on `127.0.0.1` and/or `::1`. No option exposes
  remote binding, and none is added "for testing".
- Every endpoint carrying recording data requires an unguessable per-launch capability. Loopback
  binding alone is not treated as sufficient protection.
- An unauthenticated response contains no recording metadata and no payload — not a session id,
  not a record count, not a schema version, not an error quoting a record.
- A minimal inert page and its assets are bundled with the binary, so no network access is
  required at any point. No CDN, external font, telemetry, upload path, or third-party asset.
- Restrictive caching, content-type, referrer, framing, and script policies are set, and no
  recording data is persisted by the browser: no service worker, no local storage, no analytics,
  no intentional caching.
- Recording-derived content cannot become executable markup on any path the server controls.
- Request logging cannot disclose payloads or the capability. If the exposure is hard to bound,
  log nothing.
- If opening a browser automatically fails, the server keeps serving and prints the URL.
- The server terminates with the foreground process, leaving no listener and no state directory
  behind.
- Tests cover: authorization, including that an unauthenticated request yields nothing about the
  recording; loopback binding; path handling for the `--recording` argument; `--no-open`
  suppressing any browser launch; hostile payload strings surviving as text; response headers;
  and the absence of writes and of outbound requests.
- No general web framework or async runtime is adopted unless the implementation demonstrates the
  added machinery is materially smaller and safer than a narrow alternative, and says so in the
  result. The kernel currently carries three runtime dependencies; adding a tree of them to serve
  one page to one browser needs an argument, not a convention.
- `scripts/check.sh` passes, the slice is committed, and dragons 1–3 stay open.
