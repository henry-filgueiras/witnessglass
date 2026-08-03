---
id: tsk_01KZ2CEB6K6GZAQ0FJ6N4AGF16
sequence: 9
kind: task
status: closed
sprint: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
created: 2026-08-02
closed: 2026-08-02
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
## Result

`witnessglass view --recording <PATH> [--no-open]` exists. It replays and projects one
recording, holds the serialized projection as a single immutable in-memory snapshot, binds an
operating-system-selected port on `127.0.0.1`, and serves that snapshot behind an unguessable
per-launch capability until the process is ended. `src/view.rs` is the whole of it, plus two
bundled assets and one CLI verb. **No dependency was added**: `Cargo.toml` and `Cargo.lock` are
unchanged, and the crate still carries exactly three runtime dependencies.

### 1. On not adopting a web framework or an async runtime

The acceptance criteria asked for an argument rather than a convention, so: the narrow
alternative won, and by a wide margin.

The whole server is `std::net::TcpListener`, a bounded thread-per-connection loop, a request-head
parser that reads a method and a target and discards everything else, and four routes. The
routing table has three entries and one of them is a stylesheet. There is no body parsing, no
content negotiation, no compression, no keep-alive, no cookies, no sessions, no middleware
stack, and no configuration. Every response is one of four statuses with a fixed header block.

An async runtime buys concurrency this workload does not have — one browser, one page, one
stylesheet, one JSON fetch — and costs a large dependency tree that this process would then be
serving unredacted recording data through. A web framework buys routing, extraction, and
middleware, all of which are here already in less code than the framework's own configuration
would take. Both would also make the security posture harder to state: the reason this build can
say "nothing is logged" and "no header is set that is not in this one array" is that there is no
layer underneath it doing anything on its own initiative.

The one thing a framework would genuinely have supplied is a battle-tested HTTP parser. That
matters when the input is hostile and arbitrary. Here the input is a browser on the same
machine that already has the recording on it, the parser reads at most 8 KiB with a hard timeout,
and it extracts exactly two strings. The trade is worth making, and it is stated here so that a
later slice needing real HTTP surface can revisit it rather than inherit it.

### 2. The snapshot is taken once

`Snapshot::load` calls `replay_file`, then `inspect`, then serializes the projection to a
`String`, and then the file is closed and never named again. `Snapshot` holds no path, no file
handle, and no way back. The `Viewer` holds a `Snapshot`, the two bundled assets with the
capability substituted in, and an in-flight counter — nothing else.

This is the shape Henry approved for task:8's borrow-versus-own question: serialize once at
startup rather than recompute per request. It makes "immutable snapshot" a property of the code
rather than a promise about it, and the test suite checks it the blunt way — rewrite the
recording underneath a running viewer, request again, assert the bytes are identical; then delete
the recording entirely and assert the viewer still serves the same thing.

Ordering is part of the contract. Replay, project, and bind all happen before a browser is
launched, so a corrupt recording fails at a terminal rather than in a tab. A truncated recording
succeeds and is served, with the truncation carried into the projection as `ValidPrefix` scope,
and the command says so on stderr before it serves.

### 3. Security posture

**Binding.** `TcpListener::bind((Ipv4Addr::LOCALHOST, 0))`, once, with no parameter. There is no
`--host`, no `--port`, no environment variable, and no test-only escape hatch. A test asserts
there is exactly one `bind` call in the module, that it is that one, and that the string
`0.0.0.0` appears nowhere in it.

**Capability.** 256 bits from `/dev/urandom`, hex-encoded to 64 characters, fresh per launch.
There is no fallback: on a platform with no wired-up random source the command refuses to start
rather than inventing a weaker secret, because something that looks like a secret and is not is
worse than a missing feature. `Capability` implements no `Display` and its `Debug` renders
`Capability(<redacted>)`, so it cannot reach a terminal or an error message by accident; getting
at it takes a deliberate `as_str()`. Comparison is over the full byte string with no early
return.

**Authorization before routing.** Every route requires the capability, including the page and the
stylesheet. An unauthorized request and an authorized request for a path that does not exist get
byte-identical responses: `404`, `Content-Type: text/plain`, body `not found\n`. A test asserts
that equality directly, and separately asserts that no unauthenticated response anywhere contains
the session id, a tool id, a tool name, a record count, a schema version, or any event kind.

**Headers**, on every response including the 404s:

```
Cache-Control: no-store, no-cache, must-revalidate, max-age=0
Pragma: no-cache
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Referrer-Policy: no-referrer
Content-Security-Policy: default-src 'none'; script-src 'none'; style-src 'self';
                         connect-src 'self'; img-src 'none'; font-src 'none';
                         object-src 'none'; base-uri 'none'; form-action 'none';
                         frame-ancestors 'none'
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Resource-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
Connection: close
```

No `Server` header and no `Date` header: neither is useful to a browser on the same machine and
both are disclosure for nothing. No cookie is ever set, so there is nothing for a browser to
persist. `script-src 'none'` is accurate for what this build serves rather than pre-widened for
what a later one might; task:10 will have to relax it deliberately, which is the point of writing
it tight now.

**Nothing recording-derived becomes markup.** The bundled page contains no recording data at all
— a test asserts the served page contains neither a hostile payload string nor even the tool id
from the recording behind it. Recording data leaves on exactly one route, as
`application/json` with `nosniff`. The hostile-payload test puts
`</script><img src=x onerror=alert(1)><svg onload=alert(2)>"'` + a backtick into both a reported
intent and a tool name, and asserts it comes back through the JSON **intact** — the payload must
survive semantically, not be sanitized away, because sanitizing evidence is its own defect.

**No filesystem mapping.** Both assets are `include_str!`-compiled into the binary. The server
never turns a request path into a filesystem path, so path traversal has nothing to traverse, and
no network access is needed at any point — a test greps both served assets for `http://`,
`https://`, protocol-relative URLs, `<script`, `serviceWorker`, `localStorage`, `sessionStorage`,
`indexedDB`, and inline event handlers.

**Nothing is logged.** No request line, no path, no query, no header, no timing, no status. The
URL carries the capability in its query string and the responses carry evidence; the only amount
of request logging whose exposure is easy to bound is none. The command prints four lines to
stderr at startup — record count, an incompleteness warning if applicable, the
"not redacted, not safe to share" line, and the URL — and then nothing until it dies.

**Only reading verbs.** `GET` and `HEAD`. Anything else gets a `405` with an empty body and is
told nothing about whether the path it named is real.

**Bounded work per connection.** At most 8 KiB of request head, a 10-second read and write
timeout, and at most 64 connections in flight. The bound was added after noticing that a strictly
sequential accept loop would let one speculative idle browser connection hold the page for a
whole read timeout; the thread-per-connection loop fixes that, and the counter exists because
"loopback only" is not the same as "nothing local can misbehave" and an unbounded spawn on an
accept loop is a thread bomb with a polite name.

### 4. Not a daemon

Foreground, no fork, no background mode, no PID file, no state directory, no file watching, no
tailing, no refresh. It dies with the command that started it. A test spawns the real binary,
confirms it is serving, kills it, and then polls until the port refuses connections — the
listener does not outlive the process. Another test asserts the viewer creates nothing in the
directory beside the recording and leaves the recording byte-for-byte unchanged.

`--no-open` is the absence of a call rather than a flag threaded into the server:
`open_in_browser` is a free function the CLI calls after printing the URL, and a test asserts
structurally that no process is spawned anywhere in the binding or serving path. The URL is
printed **before** any launch attempt, so a failed launch prints a note and keeps serving, which
is what a remote terminal or a headless host gets.

### 5. Tests and gate

`tests/view.rs`, 19 tests, all synthetic. They speak raw HTTP over a `TcpStream` rather than
through a client library, because the exact bytes — which status, which headers, which body — are
what is under test. Three of them drive the real compiled binary.

Covering the criteria: authorization on all four routes and the disclosure-free 404; unauthorized
and unknown-path responses being indistinguishable; capability shape, freshness, and redacted
`Debug`; loopback-only binding on an OS-selected port; the URL's form; every required header on
every route; `GET`/`HEAD` allowed and `POST` refused; hostile payloads surviving as JSON text and
never reaching the page; the snapshot surviving a rewrite and a delete of its source; no writes
and no files created; a corrupt recording failing before anything binds; a missing path and a
directory both failing without a directory being scanned; a truncated recording serving with
`ValidPrefix` scope; an empty recording serving a projection that declares no schema; the bundled
assets referencing nothing remote and containing no script or storage API; the real command
serving under `--no-open` and dying with its process; the real command refusing a corrupt
recording without printing a URL; and `view` rejecting a missing `--recording` as well as
invented `--host` and `--port` flags.

`./scripts/check.sh`, final run:

```
==> shell syntax
==> cargo fmt
==> cargo clippy
==> cargo test
    0, 0, 11, 18, 27, 2, 18, 35, 9, 8, 7, 19 passed; 0 failed
==> scarp doctor
doctor: 25 artifact(s) checked, no problems found
==> all checks passed
```

154 tests, up from 135. The 19 new ones are the viewer; the other 135 are unchanged and still
pass.

A manual smoke test was run against a **synthetic** four-record recording written to a scratch
directory outside the repository, confirming the startup lines, the 404 without a capability, the
full header block, and the projection over the wire. No real recording was read, listed, copied,
or opened by this task, and `.witnessglass/` was not touched.

### 6. What changed outside the task

`README.md` was inaccurate in two ways after task:8 and would have been in a third after this
one. It said projections do not exist, that no command accepts `view`, and it described the
viewer as unbuilt. It now lists the projection and the `view` command under what works, gains a
"Viewing a recording in a browser" section stating the security posture and the snapshot
semantics in the terms above, and says plainly that the browser half — the HUD, the map, the
ledger, the inspector — does not exist yet. The non-goals section now points at decision:6.

The "what does not exist" line still leads with redaction, and the new section still ends with
"rendering is not redacting". dragon:2 is unchanged and unweakened by a word.

### Scarp desire paths

**idea:1 recurred, for the ninth time.** Result written to a temporary file, appended with a
shell redirect, then `scarp close task:9`. Nine for nine. The count is the only new information.

**No new idea is filed.** Scarp was not in the way of this task at any point. The friction was
entirely in deciding how much HTTP to write by hand, which is a WitnessGlass judgement call and
not an affordance anything could have supplied.

### What task:10 inherits, and one thing it must decide

The projection is at `/projection.json` behind the capability, and the page is at `/` with the
capability already substituted into both of its links. task:10 replaces `src/assets/viewer.html`
and its stylesheet and adds whatever script it needs.

Two things it has to deal with deliberately rather than by accident:

- **`script-src 'none'` will have to be widened**, and it should be widened as narrowly as the
  workbench actually needs. A bundled script served from `/` is `script-src 'self'`; an inline
  bootstrap would need a nonce or a hash, and reaching for `'unsafe-inline'` because it is easier
  would undo most of what this slice bought.
- **The capability has to reach the fetch without ending up somewhere durable.** It is already in
  `location.search`, which is also the browser history and the tab title bar. Reading it from
  there is fine; writing it into storage, a cookie, or a service worker is not, and the sprint's
  "no browser persistence" criterion is the one a convenience here would break first.
