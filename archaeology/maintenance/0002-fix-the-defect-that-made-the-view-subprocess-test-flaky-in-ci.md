---
id: mnt_01KZC5YMGRDMJ1N3KWFCPEGZNC
sequence: 2
kind: maintenance
status: closed
created: 2026-08-06
closed: 2026-08-06
---

# Fix the defect that made the view subprocess test flaky in CI

## Work

A defect in the `view` test helper, not in the viewer behaviour under
test: `the_command_serves_without_opening_a_browser_and_dies_with_the_process`
fails intermittently with a connection reset at `tests/view.rs:78`.

Detected from CI rather than locally, which is the whole reason it
survived. Five failed runs on `main`, every one naming that same test and
that same line:

- 2026-08-03 `7e599f1` (run 30775213182)
- 2026-08-05 `0db56c2` (run 31044220005)
- 2026-08-05 `dbfe800` (run 31056576587)
- 2026-08-05 `9315fe1` (run 31057773133)
- 2026-08-06 `a1430a2` (run 31059511372)

The failures interleave with passing runs on neighbouring commits, so
nothing in the commits that failed is implicated — 8 of the 12 runs in
that window passed. It never reproduced locally: 80 runs of the test
under CPU load on macOS, zero failures.

Root cause. `spawn_view` reads the child's stderr only until the URL
line, then returns and drops the `BufReader`. That closes the read end of
the pipe while `run_view` still has one line left to write after the URL
— `serving in the foreground; press Ctrl-C to stop`. `eprintln!` to a
closed pipe returns EPIPE and panics, so the viewer exits 101 before it
ever reaches `serve_forever`. The test's connection, already accepted
onto the listener backlog, is reset when the process dies, and
`read_to_end` fails with ECONNRESET.

The race is the few microseconds between those two writes. That is why a
loaded CI runner hits it roughly 40% of the time and an idle laptop never
does.

Keep draining the child's stderr for its lifetime.

## Result

Fixed in `46cad16`: `spawn_view` now drains the child's stderr on a
background thread for the child's lifetime, so the pipe stays open until
the process ends.

Verified causally rather than by absence of failure, because a race that
needs a loaded runner cannot be shown fixed by watching it not happen.
Inserting a 300ms sleep between the two `eprintln!` calls in `run_view`
widens the window deterministically. With that sleep the test failed 100%
of the time before the change, reproducing the CI symptom exactly — same
test, same `tests/view.rs:78`, same `ConnectionReset` (code 54 on macOS,
104 on Linux in CI) — and passes with the sleep still in place after it.
The sleep was reverted; `scripts/check.sh` passes.

The EPIPE mechanism was confirmed on its own: closing the read end of a
spawned viewer's stderr before it writes anything exits the process 101,
in 5 of 5 trials.

Two adjacent sharp edges left alone deliberately.

`spawn_view` also pipes the child's stdout and never reads it. A child
that wrote more than a pipe buffer to stdout would block forever;
`run_view` writes nothing to stdout, so this is latent rather than live.

More interesting: the viewer dying of EPIPE because its stderr closed is
arguably a defect in the product, not only in the test. A foreground
server that stops serving because nobody is reading its log is a sharp
edge for exactly the case `--no-open` exists to serve — a remote terminal
where the invoking process may not hold stderr open. Whether `view`
should tolerate a closed stderr is a behaviour decision with a real
argument on both sides, so it is recorded here rather than settled by a
test fix.
