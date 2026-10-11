#!/bin/bash
# Weekly autonomous technique-research run.
#
# Scheduled by the launchd user agent com.mechcrate.research-weekly (Mondays
# 09:03 local; template scripts/launchd/com.mechcrate.research-weekly.plist,
# installer scripts/launchd/install-research-weekly.sh). It is a LaunchAgent and
# not a crontab entry because a cron job on macOS cannot read the login Keychain
# that holds Claude Code's OAuth credentials: every cron run from 2026-07-20 to
# 2026-09-14 failed with "Not logged in". launchd also runs a missed calendar
# slot after the Mac wakes, where cron silently skips it.
#
# Invokes headless Claude Code with the technique-research skill in autonomous
# mode. The bot owns its source: a clone at ~/.mech-crate/research-source that
# the launcher fast-forwards to origin/main before exec'ing this file, and a
# worktree of it (~/.mech-crate/research-worktree on branch research-bot-main,
# reset to origin/main every run) where the run edits and commits. No human
# checkout is read or written, whatever state it is in. The run ingests UNTRUSTED web
# content, so permissions are scoped to an explicit allowlist (no
# --dangerously-skip-permissions): repo file edits, the specific CLIs the
# pipeline needs, web fetch/search, and the mx MCP tools. Output is
# additionally PR-gated: the run opens a PR, never merges.
#
# Pause:  launchctl bootout gui/$(id -u)/com.mechcrate.research-weekly
# Run now: launchctl kickstart gui/$(id -u)/com.mechcrate.research-weekly
# Logs:   ~/.mech-crate/research-cron.log
set -u
export PATH="$HOME/.local/bin:/usr/local/bin:/opt/homebrew/bin:$PATH"
SRC="${MECH_CRATE_SOURCE:-$HOME/.mech-crate/research-source}"
# RESEARCH_WORKTREE and RESEARCH_LOG exist for the test harness
# (scripts/tests/research-weekly-outcome.sh); production uses the defaults.
WT="${RESEARCH_WORKTREE:-$HOME/.mech-crate/research-worktree}"
LOG="${RESEARCH_LOG:-$HOME/.mech-crate/research-cron.log}"
mkdir -p "$HOME/.mech-crate"
log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*" >> "$LOG"; }

log "weekly technique-research run starting"
if ! git -C "$SRC" fetch -q origin main 2>>"$LOG"; then
  log "run aborted: git fetch failed in $SRC"
  exit 1
fi
git -C "$SRC" worktree prune
if [ ! -e "$WT/.git" ]; then
  if ! git -C "$SRC" worktree add -q -B research-bot-main "$WT" origin/main 2>>"$LOG"; then
    log "run aborted: could not create worktree $WT"
    exit 1
  fi
fi
# A clean tree at origin/main every run; leftover research/* branches from
# earlier runs are harmless and stay out of the way.
if ! { git -C "$WT" checkout -q -B research-bot-main origin/main \
       && git -C "$WT" reset -q --hard origin/main \
       && git -C "$WT" clean -fdq; } 2>>"$LOG"; then
  log "run aborted: could not reset worktree $WT to origin/main"
  exit 1
fi
export MECH_CRATE_ROOT="$WT"
cd "$WT" || exit 1

# Headless print mode gives background subagents only 600 s after the model's
# turn ends and then terminates WITH EXIT 0. The 2026-09-28 run died that way
# while its web-research subagent was still working, and opened no PR. 0 means
# wait until they report (the model is then re-invoked); `timeout` is the bound.
export CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=0

RUN_OUT=$(mktemp "${TMPDIR:-/tmp}/research-weekly.XXXXXX")
timeout 7200 claude -p "Invoke the technique-research skill (Skill tool, skill: technique-research) in autonomous mode: no topic given: follow its Phase 0 (Hacker News trend pulse, then the autonomous ladder). Follow the skill exactly. The repo is \$MECH_CRATE_ROOT ($WT); branch research/<slug> off origin/main there. This run is headless: nobody will prompt you again. If you dispatch a subagent, keep working on the other steps; if you have to stop and wait for it, say so, and you will be re-invoked when it reports. The run is only finished when the PR is open: end by printing its URL on a line of its own." \
  --allowedTools "Read" "Write" "Edit" "Glob" "Grep" "Skill" "Agent" "TodoWrite" "WebSearch" "WebFetch" "ToolSearch" \
    "Bash(git:*)" "Bash(gh pr:*)" "Bash(gh issue:*)" "Bash(cargo:*)" "Bash(mx:*)" "Bash(curl:*)" "Bash(jq:*)" "Bash(date:*)" \
    "Bash(ls:*)" "Bash(cat:*)" "Bash(grep:*)" "Bash(mkdir:*)" "Bash(head:*)" "Bash(tail:*)" "Bash(wc:*)" \
    "mcp__mx__rag_context" "mcp__mx__rag_search" "mcp__mx__rag_search_category" "mcp__mx__rag_find_implementation" \
    "mcp__mx__rag_get_guidance" "mcp__mx__rag_compare_approaches" "mcp__mx__rag_find_related" "mcp__mx__rag_health" \
    "mcp__x__search_recent" "mcp__x__get_user" \
  < /dev/null 2>&1 | tee -a "$LOG" > "$RUN_OUT"
rc=${PIPESTATUS[0]}

# Exit 0 does not mean a PR exists (see above), so the outcome is read from
# what the run printed. No PR and no deliberate stop is a failed run: say so in
# the log and exit non-zero so `launchctl print` shows it.
pr=$(grep -Eo 'https://github\.com/[^/ ]+/[^/ ]+/pull/[0-9]+' "$RUN_OUT" | tail -n 1)
if [ -n "$pr" ]; then
  log "run finished (exit $rc, PR $pr)"
elif grep -Eq '(^|[^A-Za-z])FRESH([^A-Za-z]|$)|insufficient sources' "$RUN_OUT"; then
  log "run finished (exit $rc, no PR: the run reported FRESH or insufficient sources)"
else
  log "run finished WITHOUT a PR (exit $rc): failed run, read its output above"
  [ "$rc" -eq 0 ] && rc=3
fi
rm -f "$RUN_OUT"
exit $rc
