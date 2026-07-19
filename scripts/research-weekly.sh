#!/bin/bash
# Weekly autonomous technique-research run (installed in user crontab: 3 9 * * 1).
#
# Invokes headless Claude Code with the technique-research skill in autonomous
# mode. The run ingests UNTRUSTED web content, so permissions are scoped to an
# explicit allowlist (no --dangerously-skip-permissions): repo file edits, the
# specific CLIs the pipeline needs, web fetch/search, and the mx MCP tools.
# Output is additionally PR-gated — the run opens a PR, never merges.
# Pause: comment out the crontab line (crontab -e). Logs:
# ~/.mech-crate/research-cron.log
set -u
export PATH="$HOME/.local/bin:/usr/local/bin:/opt/homebrew/bin:$PATH"
REPO="${MECH_CRATE_ROOT:-$HOME/dev/dev916/mech-crate}"
LOG="$HOME/.mech-crate/research-cron.log"
cd "$REPO" || exit 1
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "[$ts] weekly technique-research run starting" >> "$LOG"
timeout 7200 claude -p "Invoke the technique-research skill (Skill tool, skill: technique-research) in autonomous mode: no topic given — follow its Phase 0 autonomous ladder. Follow the skill exactly." \
  --allowedTools "Read" "Write" "Edit" "Glob" "Grep" "Skill" "Agent" "TodoWrite" "WebSearch" "WebFetch" "ToolSearch" \
    "Bash(git:*)" "Bash(gh pr:*)" "Bash(gh issue:*)" "Bash(cargo:*)" "Bash(mx:*)" "Bash(curl:*)" \
    "Bash(ls:*)" "Bash(cat:*)" "Bash(grep:*)" "Bash(mkdir:*)" "Bash(head:*)" "Bash(tail:*)" \
    "mcp__mx__rag_context" "mcp__mx__rag_search" "mcp__mx__rag_search_category" "mcp__mx__rag_find_implementation" \
    "mcp__mx__rag_get_guidance" "mcp__mx__rag_compare_approaches" "mcp__mx__rag_find_related" "mcp__mx__rag_health" \
    "mcp__x__search_recent" "mcp__x__get_user" \
  >> "$LOG" 2>&1
echo "[$ts] run finished (exit $?)" >> "$LOG"
