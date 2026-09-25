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
# mode, inside a dedicated worktree (~/.mech-crate/research-worktree on branch
# research-bot-main, reset to origin/main every run) so the owner's checkout is
# never read or written, whatever state it is in. The run ingests UNTRUSTED web
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
SRC="${MECH_CRATE_SOURCE:-$HOME/dev/dev916/mech-crate}"
WT="$HOME/.mech-crate/research-worktree"
LOG="$HOME/.mech-crate/research-cron.log"
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

timeout 7200 claude -p "Invoke the technique-research skill (Skill tool, skill: technique-research) in autonomous mode: no topic given: follow its Phase 0 (Hacker News trend pulse, then the autonomous ladder). Follow the skill exactly. The repo is \$MECH_CRATE_ROOT ($WT); branch research/<slug> off origin/main there." \
  --allowedTools "Read" "Write" "Edit" "Glob" "Grep" "Skill" "Agent" "TodoWrite" "WebSearch" "WebFetch" "ToolSearch" \
    "Bash(git:*)" "Bash(gh pr:*)" "Bash(gh issue:*)" "Bash(cargo:*)" "Bash(mx:*)" "Bash(curl:*)" "Bash(jq:*)" "Bash(date:*)" \
    "Bash(ls:*)" "Bash(cat:*)" "Bash(grep:*)" "Bash(mkdir:*)" "Bash(head:*)" "Bash(tail:*)" "Bash(wc:*)" \
    "mcp__mx__rag_context" "mcp__mx__rag_search" "mcp__mx__rag_search_category" "mcp__mx__rag_find_implementation" \
    "mcp__mx__rag_get_guidance" "mcp__mx__rag_compare_approaches" "mcp__mx__rag_find_related" "mcp__mx__rag_health" \
    "mcp__x__search_recent" "mcp__x__get_user" \
  >> "$LOG" 2>&1
rc=$?
log "run finished (exit $rc)"
exit $rc
