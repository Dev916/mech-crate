#!/bin/bash
# Weekly autonomous technique-research run (installed in user crontab: 3 9 * * 1).
#
# Invokes headless Claude Code with the technique-research skill in autonomous
# mode. Output is PR-gated — the run can only open a PR, never merge, so the
# blast radius of --dangerously-skip-permissions is bounded to a reviewable
# branch. Pause: comment out the crontab line (crontab -e). Logs:
# ~/.mech-crate/research-cron.log
set -u
export PATH="$HOME/.local/bin:/usr/local/bin:/opt/homebrew/bin:$PATH"
REPO="${MECH_CRATE_ROOT:-$HOME/dev/dev916/mech-crate}"
LOG="$HOME/.mech-crate/research-cron.log"
cd "$REPO" || exit 1
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "[$ts] weekly technique-research run starting" >> "$LOG"
timeout 7200 claude -p "Invoke the technique-research skill (Skill tool, skill: technique-research) in autonomous mode: no topic given — follow its Phase 0 autonomous ladder. Follow the skill exactly." \
  --dangerously-skip-permissions >> "$LOG" 2>&1
echo "[$ts] run finished (exit $?)" >> "$LOG"
