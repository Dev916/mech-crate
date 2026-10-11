#!/bin/bash
# Install (or reinstall) the weekly technique-research LaunchAgent for the
# current user. Idempotent.
#
#   scripts/launchd/install-research-weekly.sh [SOURCE]
#
# SOURCE is the clone the bot runs from (default ~/.mech-crate/research-source).
# It is created from this repo's origin URL when missing, and the agent's
# launcher fast-forwards it to origin/main before every run, so it must be a
# clone nobody edits by hand. The installer also retires the crontab entry the
# agent replaces, and points ~/.claude/skills/technique-research at the clone's
# copy of the skill when that path is absent or already a symlink, so the skill
# the headless run loads is the one merged on main.
#
# Afterwards:
#   launchctl print gui/$(id -u)/com.mechcrate.research-weekly | head -20   # state, next run
#   launchctl kickstart gui/$(id -u)/com.mechcrate.research-weekly          # run now
#   launchctl bootout gui/$(id -u)/com.mechcrate.research-weekly            # pause
set -euo pipefail

LABEL=com.mechcrate.research-weekly
HERE=$(cd "$(dirname "$0")/../.." && pwd -P)
SOURCE="${1:-$HOME/.mech-crate/research-source}"
AGENTS="$HOME/Library/LaunchAgents"
PLIST="$AGENTS/$LABEL.plist"
DOMAIN="gui/$(id -u)"

mkdir -p "$AGENTS" "$HOME/.mech-crate"
if [ ! -e "$SOURCE/.git" ]; then
  url=$(git -C "$HERE" remote get-url origin)
  echo "cloning $url -> $SOURCE"
  git clone -q --branch main "$url" "$SOURCE"
fi
git -C "$SOURCE" fetch -q origin main
git -C "$SOURCE" reset -q --hard origin/main
TEMPLATE="$SOURCE/scripts/launchd/$LABEL.plist"
[ -f "$TEMPLATE" ] || { echo "template missing in $SOURCE (is main up to date?): $TEMPLATE" >&2; exit 1; }
[ -x "$SOURCE/scripts/research-weekly.sh" ] || { echo "$SOURCE/scripts/research-weekly.sh is not executable" >&2; exit 1; }

sed -e "s#__HOME__#$HOME#g" -e "s#__SOURCE__#$SOURCE#g" "$TEMPLATE" > "$PLIST"
plutil -lint "$PLIST" >/dev/null

# Reload so an edited template takes effect.
launchctl bootout "$DOMAIN/$LABEL" 2>/dev/null || true
launchctl bootstrap "$DOMAIN" "$PLIST"

# The crontab entry this agent replaces must not fire alongside it.
if crontab -l 2>/dev/null | grep -q 'scripts/research-weekly.sh'; then
  crontab -l | grep -v 'scripts/research-weekly.sh' | crontab -
  echo "removed the research-weekly crontab line"
fi

# The headless run loads the skill from ~/.claude/skills, not from the repo.
SKILL_LINK="$HOME/.claude/skills/technique-research"
if [ -L "$SKILL_LINK" ] || [ ! -e "$SKILL_LINK" ]; then
  mkdir -p "$HOME/.claude/skills"
  ln -sfn "$SOURCE/skills/technique-research" "$SKILL_LINK"
  echo "linked $SKILL_LINK -> $SOURCE/skills/technique-research"
else
  echo "note: $SKILL_LINK is a plain directory; keep it in sync with skills/technique-research or replace it with a symlink to $SOURCE/skills/technique-research"
fi

echo "installed $LABEL (source $SOURCE)"
launchctl print "$DOMAIN/$LABEL" | grep -E '^\s*(state|last exit code|run interval)' || true
