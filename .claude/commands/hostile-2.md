---
description: "HISTORICAL, superseded: pass 2's turn 2. Kept as a record; do not run as written."
---

**HISTORICAL — pass 2's turn 2, run 2026-08-03 on Claude Code 2.1.220. Superseded, first by
`/hostile-3` and then by that command's own miss. Do not run this as written.**

Both of its two human-operated steps failed for reasons now understood, and neither failure is
fixed here: step 3's denial did not prompt because the session ran in `auto` permission mode,
and step 4's bare `sleep` is refused by the harness before dispatch, so it left no trace in any
hook. Its results are in dragon:1 and dragon:3 and in `docs/claude-adapter.md` §3.2.

**If this command is invoked**: say that it is a superseded protocol and what its known misses
are, and do not begin the steps unless the operator confirms they want the historical run
reproduced anyway.

<!--
OPERATOR NOTE — for the human running this, not an instruction to the agent.

The note below is pass 2's, preserved as written. It was accurate for pass 2 and is not
accurate as instructions today.

Submit this as a SECOND turn in the SAME session that ran /hostile-1. Do not
restart, do not /clear. A new session gets a new session_id and the prompt_id
comparison is lost.

You have two jobs during this turn, and neither can be scripted:

  STEP 3 — the agent will ask permission to delete a directory. DENY it.
           Choose "no". This measures whether PermissionDenied fires for an
           interactive refusal, which decision:4 recorded as unmeasured. If no
           prompt appears and it just runs, that is itself the finding — note it.

  STEP 4 — the agent will start `sleep 120`. Wait about five seconds, then
           INTERRUPT it (Esc). This is the only way to observe `interrupted`,
           which has never been seen populated.

Interrupting ends the turn. Everything before it has already been recorded, so
that is fine and expected.

Afterwards:
  scripts/disarm.sh
  scripts/probe.sh show          # payload-quiet: key names only
  ./scripts/check-recording.sh .witnessglass/recordings/<session-id>.ndjson
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

4. Run exactly this: `sleep 120`

   **Expect to be interrupted.** If you are interrupted, do not retry it and do
   not start anything else. Simply stop.

If you somehow reach the end of step 4 without interruption, reply with one line
saying so, and stop.
