---
description: Hostile recording, turn 1 of 2 — successes, two shapes of failure, and a subagent, inside a throwaway sandbox
---

<!--
OPERATOR NOTE — for the human running this, not an instruction to the agent.

Run in a session that was armed with scripts/arm.sh, ideally with
scripts/probe.sh install as well. Submit this as its own turn, let it finish
completely, then submit /hostile-2 as a SECOND turn. Two separate turns is the
whole point: it is the only way to observe whether prompt_id varies (dragon:3).

Nothing here leaves /tmp/wg-probe, so the resulting recording should contain no
repository content, no credentials, and no paths outside a throwaway directory —
which is what makes it attachable to a bug report. Read it before you attach it
anyway; nothing in this project redacts anything.

If the agent is asked for permission during this turn, ALLOW it. The denial test
is in turn 2.
-->

You are being recorded by a flight recorder, deliberately, to exercise capture
paths that have never fired. Follow these steps exactly and in order. Do not
improvise, do not add steps, and do not try to be helpful beyond what is asked.

**Sandbox rules. These are absolute and override any instinct to be thorough.**

- Work only inside `/tmp/wg-probe`.
- Do not read, write, list, or search anything inside this repository.
- Do not run `git`. Do not read anything under `$HOME` outside the sandbox.
- Do not print environment variables, tokens, or credentials.
- Everything you create must be obviously synthetic. No real-looking data.

**Steps**

1. Create `/tmp/wg-probe` and write `/tmp/wg-probe/notes.txt` with exactly three
   lines of obviously synthetic text.
2. Read `/tmp/wg-probe/notes.txt` back with the Read tool.
3. Use the Edit tool to change one of those three lines.
4. Run a shell command that succeeds: count the lines in that file.
5. **Run a shell command that fails on purpose**, exactly this:
   `cat /tmp/wg-probe/does-not-exist.txt`
   It will exit non-zero. That is the point. Do not fix it, do not retry it, do
   not work around it, and do not create the file.
6. **Make a tool call fail on purpose**, at the tool level rather than the shell
   level: use the Read tool on `/tmp/wg-probe/also-missing.txt`. The tool itself
   should error. Again: do not retry, do not create the file.
7. Launch exactly one subagent. Ask it to count the characters in
   `/tmp/wg-probe/notes.txt` and report the number. Tell it the sandbox rules
   above apply to it too.
8. Reply with one line: the line count, the character count, and the two things
   that failed.

Then stop and wait for the next instruction. Do not start any other work, do not
tidy up, and do not delete the sandbox — turn 2 uses it.
