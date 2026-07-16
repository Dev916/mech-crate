---
name: writing-devloop-plans
description: 'Use when writing an implementation plan for a project that will be executed with the devloop skill. Produces a standard writing-plans plan with one augmentation — every task has an explicit Acceptance Criteria section up front listing observable facts the devloop subagent verifies via the appropriate toolkit (playwright, xcodebuildmcp, api, or cli). Triggers: "write a devloop plan", "plan for devloop", "plan with acceptance criteria".'
---

# Writing Devloop Plans

Companion to `superpowers:writing-plans` for plans that will be executed by the `devloop` skill. Produces the same format as writing-plans PLUS a per-task Acceptance Criteria section that devloop's per-task subagent uses to drive Playwright observation.

**Announce at start:** "I'm using the writing-devloop-plans skill to create a devloop-compatible implementation plan."

## Why this exists

`superpowers:writing-plans` produces tasks with TDD steps and step-level `- [ ]` checkboxes. Those work great for code-correctness verification (test passes means code is right), but devloop's per-task subagent ALSO runs Playwright observe passes to verify user-visible behavior. Without explicit acceptance criteria, the subagent has to infer which parts of the spec apply to this task — which can go wrong on larger plans.

This skill closes that gap by requiring each task to state its user-visible acceptance criteria up front, before the TDD steps.

## Process

1. **Invoke `superpowers:writing-plans` to produce the base plan.** Follow it exactly — spec coverage, bite-sized tasks, code in every step, the whole thing. The base plan is the source of truth for file paths, TDD steps, and commit structure.

   **Consult the techniques corpus while writing the base plan.** Before drafting tasks, call `mcp__mx__rag_context` with a 1–2 sentence description of the feature and its stack (plus `language` when obvious). Weave returned techniques into task design. When a task directly applies one, add an `**Apply:** <source doc path> — <technique>` line directly below that task's Acceptance Criteria so the executing subagent inherits the reference. If the corpus is offline (`mcp__mx__rag_health`) or returns nothing relevant, note it and continue — never block planning on it.

2. **Augment each task block** by inserting this section DIRECTLY BELOW the `### Task N:` heading and BEFORE the `**Files:**` section:

   ```markdown
   **Acceptance Criteria (user-visible, Playwright-observable):**
   - <fact 1 about DOM structure, route content, or visible state>
   - <fact 2>
   - <fact 3 if flow-based: expected post-action state>
   ```

3. **Rules for writing acceptance criteria:**
   - **Observable:** something Playwright can verify via DOM snapshot, console capture, or user-flow interaction. NOT code-level ("the test passes") — user-level ("visiting `/about` renders an `<h1>` containing the app name").
   - **Scoped to this task:** only what becomes true after this task is done. Don't duplicate criteria across tasks.
   - **Derived from the spec:** every criterion must trace back to something explicit in the spec. If you cannot ground it in the spec, the spec has a gap — stop and return to brainstorming.
   - **At least one, usually 2–4 per task.** Zero = the task doesn't produce user-visible change (rare for web-app plans; if so, note why in a one-liner). More than ~5 = the task is too big, break it up.
   - **Interactive vs static:** if the task involves user actions (click, submit, navigate), phrase at least one criterion as the expected post-action state ("after submitting with valid email, user is redirected to `/dashboard`"). Otherwise criteria describe the static rendered state.
   - **Toolkit hint:** if the task is clearly API or CLI rather than browser-observable, phrase criteria in terms the right toolkit can check:
     - API criteria: "POST /api/users returns 201 with JSON body containing `id` field"
     - CLI criteria: "running `mytool --version` exits 0 and prints a semver string to stdout"
     - Optionally add an explicit `**Verify via:** api` or `**Verify via:** cli` tag below the criteria heading to remove ambiguity for devloop's classifier.

4. **Plan header additions:** after the goal/architecture/stack fields the base writing-plans skill produces, add this line:

   ```markdown
   **Compatible with:** devloop skill v0.1+
   ```

   This is an unambiguous flag so devloop's orchestrator and per-task subagents can tell the plan was written with acceptance criteria present.

5. **Self-review.** For each task:
   - Re-read its Acceptance Criteria and the corresponding spec section.
   - Can you point to the spec line that justifies each criterion? If not, remove the criterion or flag the spec gap.
   - Does every criterion describe a user-visible fact, not a code-level one? ("test passes" = reject. "page renders header" = accept.)
   - Is the task still bite-sized (≤5 criteria)? If it blew past 5, split into multiple tasks.

## Example task block

```markdown
### Task 3: /about page renders author paragraph

**Acceptance Criteria (user-visible, Playwright-observable):**
- Navigating to `/about` returns a 200 and renders a page.
- The page contains an `<h1>` with the exact text "About".
- The page contains a `<p>` element with non-empty text content describing the author.
- No console errors on page load.
- No failed network requests on page load.

**Files:**
- Create: `src/routes/about.tsx`
- Test: `tests/routes/about.test.tsx`

- [ ] **Step 1: Write the failing test**

...rest of task follows the standard writing-plans template...
```

## Output location

Same as writing-plans: `docs/superpowers/plans/YYYY-MM-DD-<feature-name>.md`. User preferences override this default.

## Handoff

Once saved, tell the user: "Plan complete and saved to `<path>`. Compatible with devloop. Ready to execute with `/devloop`."

Do NOT auto-invoke devloop — the user runs it when ready.

## When NOT to use this skill

- Non-web projects (CLI tools, libraries, data pipelines) — use plain `superpowers:writing-plans`. Devloop is web-app only.
- Plans that will be executed by a human or plain `executing-plans` — the acceptance-criteria overhead has no payoff without a browser-verification loop.
