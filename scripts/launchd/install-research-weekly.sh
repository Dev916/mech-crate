#!/bin/bash
# Install (or reinstall) the weekly technique-research LaunchAgent for the
# current user, and retire the crontab entry it replaces. Idempotent.
#
#   scripts/launchd/install-research-weekly.sh
#
# Afterwards:
#   launchctl print gui/$(id -u)/com.mechcrate.research-weekly | head -20   # state, next run
#   launchctl kickstart gui/$(id -u)/com.mechcrate.research-weekly          # run now
#   launchctl bootout gui/$(id -u)/com.mechcrate.research-weekly            # pause
set -euo pipefail

LABEL=com.mechcrate.research-weekly
REPO=$(cd "$(dirname "$0")/../.." && pwd -P)
TEMPLATE="$REPO/scripts/launchd/$LABEL.plist"
AGENTS="$HOME/Library/LaunchAgents"
PLIST="$AGENTS/$LABEL.plist"
DOMAIN="gui/$(id -u)"

[ -f "$TEMPLATE" ] || { echo "template missing: $TEMPLATE" >&2; exit 1; }
[ -x "$REPO/scripts/research-weekly.sh" ] || { echo "scripts/research-weekly.sh is not executable" >&2; exit 1; }

mkdir -p "$AGENTS" "$HOME/.mech-crate"
sed -e "s#__HOME__#$HOME#g" -e "s#__REPO__#$REPO#g" "$TEMPLATE" > "$PLIST"
plutil -lint "$PLIST" >/dev/null

# Reload so an edited template or a moved repo takes effect.
launchctl bootout "$DOMAIN/$LABEL" 2>/dev/null || true
launchctl bootstrap "$DOMAIN" "$PLIST"

# The crontab entry this agent replaces must not fire alongside it.
if crontab -l 2>/dev/null | grep -q 'scripts/research-weekly.sh'; then
  crontab -l | grep -v 'scripts/research-weekly.sh' | crontab -
  echo "removed the research-weekly crontab line"
fi

echo "installed $LABEL from $PLIST"
launchctl print "$DOMAIN/$LABEL" | grep -E '^\s*(state|last exit code|program|run interval)' || true
