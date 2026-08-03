---
description: Hostile recording, turn 2 of 2 — a permission denial and an interrupted call, in the same session as turn 1
---

<!--
OPERATOR NOTE — for the human running this, not an instruction to the agent.

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
