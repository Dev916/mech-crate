#!/bin/bash
# Outcome detection in scripts/research-weekly.sh, driven with a stub `claude`.
# A headless run can exit 0 without having opened a PR; the script must tell
# the three endings apart from what the run printed.
#
#   scripts/tests/research-weekly-outcome.sh        # exits non-zero on failure
set -u
REPO=$(cd "$(dirname "$0")/../.." && pwd -P)
T=$(mktemp -d "${TMPDIR:-/tmp}/rw-outcome.XXXXXX")
trap 'rm -rf "$T"' EXIT
fail=0

# A throwaway "origin" with a main branch, and a clone of it to act as $SRC.
git init -q -b main "$T/origin"
git -C "$T/origin" -c user.name=t -c user.email=t@example.invalid commit -q --allow-empty -m init
git clone -q "$T/origin" "$T/src"

mkdir -p "$T/bin"
cat > "$T/bin/claude" <<'STUB'
#!/bin/bash
cat "$STUB_OUTPUT_FILE"
exit "${STUB_EXIT:-0}"
STUB
chmod +x "$T/bin/claude"

# Every case shares one worktree, as production does: the first run creates it
# and the later ones take the reset path.
run_case() { # name, stub text, stub exit, expected exit, expected log pattern
  local name=$1 text=$2 stub_exit=$3 want_rc=$4 want_log=$5
  local log="$T/$name.log"
  printf '%s\n' "$text" > "$T/$name.out"
  HOME="$T/home" PATH="$T/bin:$PATH" MECH_CRATE_SOURCE="$T/src" \
    RESEARCH_WORKTREE="$T/wt" RESEARCH_LOG="$log" \
    STUB_OUTPUT_FILE="$T/$name.out" STUB_EXIT="$stub_exit" \
    bash "$REPO/scripts/research-weekly.sh"
  local rc=$?
  if [ "$rc" -eq "$want_rc" ] && grep -Eq "$want_log" "$log"; then
    echo "ok   $name (exit $rc)"
  else
    echo "FAIL $name: exit $rc (want $want_rc); last log line: $(tail -n 1 "$log")"
    fail=1
  fi
}

mkdir -p "$T/home"
run_case pr "Opened https://github.com/Dev916/mech-crate/pull/123 for review." 0 0 \
  'run finished \(exit 0, PR https://github.com/Dev916/mech-crate/pull/123\)'
run_case stalled "Background tasks still running after 600s; terminating.
I'll continue with authoring as soon as the subagent reports." 0 3 \
  'run finished WITHOUT a PR \(exit 0\)'
run_case fresh "Coverage verdict: FRESH. Logged a no-op row and stopped." 0 0 \
  'no PR: the run reported FRESH or insufficient sources'
run_case crashed "Not logged in" 1 1 'run finished WITHOUT a PR \(exit 1\)'
# "refresh" must not be read as the FRESH verdict.
run_case refresh-word "Could not refresh the index; giving up." 0 3 'run finished WITHOUT a PR'

exit $fail
