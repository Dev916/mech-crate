# Devloop Per-Task Subagent Prompt

The orchestrator dispatches one subagent per plan task using this template. Substitute the `{{...}}` placeholders before sending.

---

## Prompt template

```
You are executing one task from an implementation plan with live visual verification. You are running in an isolated context — the orchestrator (main session) sees only the JSON you return at the end.

CONTEXT:
- Spec excerpt (relevant sections): {{spec_excerpt}}
- Plan task (verbatim block from the plan, including acceptance criteria):

  {{plan_task_block}}

- Project root (absolute path): {{project_root}}
- Project structure: {{project_structure}}   (one of: docker | xcode | standalone)
- Verify toolkit: {{verify_toolkit}}         (one of: playwright | xcodebuildmcp | api | cli)
- Target URL: {{target_url}}                 (URL for docker/standalone-api; "simulator" for xcode; "n/a" for cli-only)
- Binary path: {{binary_path}}               (standalone CLI only; empty otherwise)
- Observe mode: {{observe_mode}}             (static | interactive for playwright/xcodebuildmcp; "n/a" for api/cli)
- Run scratch directory (write evidence here): {{scratch_dir}}

REFERENCES YOU MUST READ BEFORE STARTING (read the one matching your verify_toolkit):

If verify_toolkit = playwright:
- ~/.claude/skills/devloop/references/observe-modes.md

If verify_toolkit = xcodebuildmcp:
- ~/.claude/skills/devloop/references/xcode-observe-modes.md
- ~/.claude/skills/devloop/references/xcodebuildmcp-reference.md

If verify_toolkit = api:
- ~/.claude/skills/devloop/references/api-observe-modes.md

If verify_toolkit = cli:
- ~/.claude/skills/devloop/references/cli-observe-modes.md

If project_structure = docker (for any toolkit):
- ~/.claude/skills/devloop/references/mx-cli-reference.md

Always read:
- ~/.claude/skills/devloop/examples/sample-task-result.json   (the exact JSON shape you must return)

YOUR JOB:

1. DERIVE ACCEPTANCE CRITERIA (before any code work).
   - If the plan task block ALREADY contains a section titled "Acceptance Criteria" (or equivalent), use those verbatim — skip to step 2.
   - Otherwise, read the spec excerpt and the plan task, then write a short bulleted list of user-visible, observable facts that must be true after this task is complete. Observable means something your verify_toolkit can check:
     - playwright: DOM snapshot, console capture, user-flow interaction
     - xcodebuildmcp: simulator screenshot, UI automation tap/swipe
     - api: HTTP response status, headers, body fields
     - cli: stdout content, stderr content, exit code, file effects
   - Every criterion must trace back to something explicit in the spec. If you cannot ground a criterion in the spec, do not invent it.
   - If the spec + task together do not give you enough information to state at least one observable criterion, STOP and return status="fail" with summary="insufficient acceptance criteria derivable from spec+task; recommend the user re-run writing-devloop-plans or add criteria manually". Do NOT guess.
   - Aim for 2-5 criteria. Record them in your scratch notes and use THESE (not generic heuristics) as the pass/fail basis in step 5.

2. CONSULT TECHNIQUES (one call, before any code work).
   - If the task block contains an "**Apply:**" line, call mcp__mx__rag_context with that technique as working_on (limit: 3).
   - Otherwise call mcp__mx__rag_context ONCE with working_on = the task title plus a one-line summary of the acceptance criteria (limit: 3).
   - Apply returned techniques only where they clearly fit THIS task; cite the source doc path in a code comment only when the pattern would otherwise be non-obvious.
   - If the tool is unavailable, offline, or returns nothing relevant: proceed without it. Never block on the corpus, never try to repair it.

3. Use the superpowers:executing-plans skill to do the code work for THIS task only.
   Do not start work on later tasks. Do not modify the plan file itself; the orchestrator does that.

4. After each meaningful change, run a verification pass using your assigned toolkit:
   - If verify_toolkit = playwright: navigate to target URL, browser_snapshot, browser_console_messages, browser_network_requests. For interactive mode, also exercise user flows (click, fill, etc.).
   - If verify_toolkit = xcodebuildmcp: rebuild with simulator/build-and-run (if code changed), take simulator/screenshot. For interactive mode, exercise flows with ui-automation tools.
   - If verify_toolkit = api: run integration tests, then make a live curl/HTTP request against target_url with the exact request from criteria. Capture status, headers, body. Save evidence to scratch_dir as task-N-api-evidence.json.
   - If verify_toolkit = cli: run integration tests, then execute the exact command from criteria. Capture stdout, stderr, exit_code. Check file effects. Save proof to scratch_dir as task-N-cli-proof.json.

5. Compare the observation to the acceptance criteria you derived in step 1.

6. If criteria are not met:
   - If verify_toolkit = playwright: check browser console/network for errors. If 5xx responses, pull `mx logs <service> 2>&1 | tail -n 50`.
   - If verify_toolkit = xcodebuildmcp: check build output for compiler errors. If app crashes, use debugging tools.
   - If verify_toolkit = api: if live hit fails but tests pass, it's a wiring issue (route not registered, middleware). Pull server logs. If both fail, it's an implementation bug.
   - If verify_toolkit = cli: if live run fails but tests pass, likely stale binary — rebuild. If wrong output, check the code path for the specific subcommand/flag.
   - Use the superpowers:systematic-debugging skill.
   - Apply a fix. Re-verify.

7. STUCK DETECTION: If the same observation, error message, or failed assertion recurs for 3 consecutive iterations, stop iterating and return status="stuck".

8. HARD CAP: 10 iterations maximum regardless of progress. If you hit it, return status="fail".

9. SUCCESS — criteria differ by toolkit:
   - playwright: all acceptance criteria observably met AND zero console errors AND zero failed network requests → return status="pass".
   - xcodebuildmcp: all criteria met AND build succeeds with no errors AND app not crashing → return status="pass".
   - api: integration tests pass AND live hit matches all acceptance criteria (status, body, headers) AND no server errors in logs → return status="pass".
   - cli: integration tests pass AND live run exit code + stdout + file effects all match criteria AND no unexpected stderr → return status="pass".
   In all cases, include the derived/used criteria in your summary.

WHAT TO RETURN:

Your LAST message to the orchestrator must be a single fenced JSON block matching the shape in examples/sample-task-result.json. No prose around it. The orchestrator parses your last message as JSON; if parsing fails the task is treated as "fail".

The JSON must include:
- status: "pass" | "fail" | "stuck"
- iterations: integer (how many observe passes you ran)
- summary: 1-3 sentence plain-English summary of what you did and what is verified
- evidence.screenshots: list of absolute file paths (write screenshots into scratch_dir)
- evidence.console_errors: list of console error strings captured in the final iteration
- evidence.network_failures: list of {url, status} objects from the final iteration
- evidence.logs_excerpts: list of strings (only populate if you pulled mx logs)
- files_changed: list of absolute paths you wrote or edited
- next_step_hint: string with a concrete suggestion if status is "stuck" or "fail"; null on "pass"
```

## Notes for the orchestrator

- `{{spec_excerpt}}`: full spec content for v0.1 (splitting by section is a future optimization).
- `{{plan_task_block}}`: verbatim markdown block for this task (heading + steps + acceptance criteria).
- `{{project_structure}}`: `docker`, `xcode`, or `standalone`, detected in Phase 1.
- `{{verify_toolkit}}`: `playwright`, `xcodebuildmcp`, `api`, or `cli`, classified per-task in Phase 4.3.1.
- `{{target_url}}`: for docker (web/api), the Traefik URL. For standalone API, localhost:port. For xcode, "simulator". For CLI-only, "n/a".
- `{{binary_path}}`: for standalone CLI tasks, the path to the built binary. Empty for all other toolkits.
- `{{observe_mode}}`: `static`/`interactive` for playwright/xcodebuildmcp. `n/a` for api/cli.
- `{{scratch_dir}}`: created once per devloop run at `/tmp/devloop-run-<ISO-date>/`.
