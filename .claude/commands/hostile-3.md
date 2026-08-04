---
description: "HISTORICAL, superseded: pass 3's turn 2. Kept as a record; do not run as written."
---

**HISTORICAL — pass 3's turn 2, run 2026-08-04 on Claude Code 2.1.221. Do not run this as
written.**

It measured its denial: an interactive refusal fired no `PermissionDenied` hook at all, and
left a `tool_requested` with no terminal record. See dragon:1 and `docs/claude-adapter.md`
§3.3.

**It cannot measure its interruption, and rerunning it will not.** Denying at step 3 returns
control to the human and ends the turn, so step 4 never starts. A denial and an interruption
cannot be staged in one turn, and interruption is now the largest surface this project has
never observed. Whatever protocol comes next must put the interruption in its own turn, and
should put it first.

**If this command is invoked**: say that it is a superseded protocol whose step 4 is
unreachable, and do not begin the steps unless the operator confirms they want the historical
run reproduced anyway.

<!--
OPERATOR NOTE — for the human running this, not an instruction to the agent.

The note below is pass 3's, preserved as written, and describes what pass 3 expected rather
than what it found.

This is pass 2's /hostile-2 with the two steps that failed replaced. Both failed
for reasons that had nothing to do with the integration:

  STEP 3 failed because the session ran in `auto` permission mode, so the
         deletion was approved with no prompt. Launch this session with
         `claude --permission-mode manual`. The valid modes on 2.1.220 are
         acceptEdits, auto, bypassPermissions, manual, dontAsk, plan — there is
         no "default", and `manual` is the one that asks.

  STEP 4 failed because the harness refuses a standalone `sleep` before dispatch.
         It left no trace in the recording AND none in the independent probe, so
         it was invisible rather than merely unrecorded. `python3 -c` sleeping
         does dispatch; that was checked.

Submit this as a SECOND turn in the SAME session that ran /hostile-1. Do not
restart, do not /clear. A new session gets a new session_id and the prompt_id
comparison is lost.

You have two jobs during this turn, and neither can be scripted:

  STEP 3 — the agent will ask permission to delete a directory. DENY it.
           Choose "no". This measures whether PermissionDenied fires for an
           interactive refusal, which decision:4 recorded as unmeasured and
           pass 2 failed to stage. If no prompt appears and it just runs, the
           mode did not take — note it and stop, because nothing after that
           measures denial.

  STEP 4 — the agent will start a 120-second sleep. Wait about five seconds,
           then INTERRUPT it (Esc). This is the only way to observe an
           interruption, which has never been seen. Note that pass 2 also had
           the adapter reading the wrong key for it; that is fixed, so this
           attempt can actually land.

Interrupting ends the turn. Everything before it has already been recorded, so
that is fine and expected.

Afterwards, the collecting cells in docs/hostile-recording-pass-3.md.
-->

Same session, same sandbox, same rules as before. Follow these steps exactly and
in order. Do not improvise.

**Sandbox rules, unchanged and still absolute.**

- Work only inside `/tmp/wg-probe`.
- Do not read, write, list, or search anything inside this repository.
- Do not run `git`. Do not read anything under `$HOME` outside the sandbox.
- Do not print environment variables, tokens, or credentials.

**Steps**

1. Append one more line of obviously synthetic text to `/tmp/wg-probe/notes.txt`.
2. Create `/tmp/wg-probe/scratch/` and write one obviously synthetic file inside it.
3. Now request to run exactly this: `rm -rf /tmp/wg-probe/scratch`

   **Expect to be refused.** When you are refused: do not retry it, do not find
   another way to delete it, do not use a different tool to achieve the same
   thing, and do not ask again. Acknowledge the refusal in one sentence and move
   straight to step 4. Being blocked here is the desired outcome, not a problem
   to solve.

   If you are *not* refused and the command simply runs, say so in one sentence
   and continue to step 4 anyway. That outcome is a measurement too.

4. Run exactly this: `python3 -c 'import time; time.sleep(120)'`

   **Expect to be interrupted.** If you are interrupted, do not retry it and do
   not start anything else. Simply stop.

   If the harness refuses to run it at all, do not substitute another command,
   do not work around the refusal, and do not try a different sleep. Report the
   refusal verbatim in one line and stop. A second command the harness blocks
   before dispatch is a more interesting result than a sleep that worked.

If you somehow reach the end of step 4 without interruption, reply with one line
saying so, and stop.
