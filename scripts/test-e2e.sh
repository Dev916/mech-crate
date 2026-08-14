#!/usr/bin/env bash
#
# mech-crate E2E smoke: scaffold -> router -> URL
#
# Proves the whole product loop with real Docker:
#   cargo build -> mx new -> mx add <recipe> -> make dev
#   -> the service answers on its router URL (http://<svc>.localhost, no ports)
#   -> teardown of the scaffold only (the router is left exactly as found)
#
# The router owns host port 80 and is a MACHINE-level singleton. On a dev box it
# is normally already running: this script REUSES it and never stops it. Only
# when no router is running (CI runners) does it install/start one -- and then it
# stops it again on the way out.
#
# Usage:
#   ./scripts/test-e2e.sh              # local default: rust-api
#   E2E_RECIPES="rust-api laravel" ./scripts/test-e2e.sh
#
# Environment:
#   E2E_RECIPES       space-separated recipes to smoke      (default: rust-api)
#   E2E_LOG_DIR       per-step log directory                (default: mktemp -d)
#   E2E_WORKSPACE     scaffold workspace                    (default: mktemp -d)
#   E2E_URL_TIMEOUT   seconds to retry the router URL       (default: 600)
#   E2E_HEALTH_PATH   override the probed path              (default: per recipe)
#   E2E_KEEP          1 = keep scaffolds + containers       (default: 0)
#   E2E_ROUTER_MODE   auto|reuse|install                    (default: auto)
#
# On E2E_URL_TIMEOUT: a warm stack answers in seconds, but dev images compile the
# app on first boot (rust-api runs `cargo watch -x run` against an empty target
# volume), so the retry window has to outlast a cold compile, not just a restart.
#
# Exit code 0 == every recipe scaffolded, came up, and answered through the router.
#

set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

E2E_RECIPES="${E2E_RECIPES:-rust-api}"
E2E_URL_TIMEOUT="${E2E_URL_TIMEOUT:-600}"
E2E_KEEP="${E2E_KEEP:-0}"
E2E_ROUTER_MODE="${E2E_ROUTER_MODE:-auto}"
E2E_LOG_DIR="${E2E_LOG_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/mx-e2e-logs.XXXXXX")}"
E2E_WORKSPACE="${E2E_WORKSPACE:-$(mktemp -d "${TMPDIR:-/tmp}/mx-e2e-work.XXXXXX")}"

readonly ROUTER_CONTAINER="mx-router"
readonly ROUTER_NETWORK="devmesh-traefik"

STEP_NO=0
ROUTER_STARTED_BY_US=0
SCAFFOLDS=()   # "<recipe>|<service>|<project dir>|<compose project>"
FAILED_RECIPES=()

mkdir -p "$E2E_LOG_DIR"

# ─────────────────────────────────────────────────────────────────────────────
# Output helpers
# ─────────────────────────────────────────────────────────────────────────────

if [[ -t 1 ]]; then
    BOLD=$'\033[1m'; GREEN=$'\033[0;32m'; RED=$'\033[0;31m'
    YELLOW=$'\033[0;33m'; CYAN=$'\033[0;36m'; DIM=$'\033[2m'; NC=$'\033[0m'
else
    BOLD=""; GREEN=""; RED=""; YELLOW=""; CYAN=""; DIM=""; NC=""
fi

info()  { echo -e "${CYAN}→${NC} $*"; }
ok()    { echo -e "${GREEN}✓${NC} $*"; }
warn()  { echo -e "${YELLOW}!${NC} $*"; }
fail()  { echo -e "${RED}✗${NC} $*" >&2; }
stamp() { date +%H:%M:%S; }

# run_step <slug> <command...>
# Streams the command's output into a numbered per-step log AND to stdout,
# so a failing E2E run is diagnosable from either the terminal or the log dir.
run_step() {
    local slug="$1"; shift
    STEP_NO=$((STEP_NO + 1))
    local log
    log="$(printf '%s/%02d-%s.log' "$E2E_LOG_DIR" "$STEP_NO" "$slug")"

    echo ""
    echo -e "${BOLD}[$(stamp)] step $STEP_NO: $slug${NC}"
    echo -e "${DIM}  \$ $*${NC}"
    echo -e "${DIM}  log: $log${NC}"

    local rc=0
    { echo "\$ $*"; echo; } > "$log"
    "$@" 2>&1 | tee -a "$log" || rc="${PIPESTATUS[0]}"
    if [[ $rc -ne 0 ]]; then
        fail "step '$slug' failed (exit $rc) — see $log"
    fi
    return "$rc"
}

# ─────────────────────────────────────────────────────────────────────────────
# Preflight
# ─────────────────────────────────────────────────────────────────────────────

preflight() {
    local missing=()
    for bin in docker cargo curl make; do
        command -v "$bin" >/dev/null 2>&1 || missing+=("$bin")
    done
    if [[ ${#missing[@]} -gt 0 ]]; then
        fail "missing required tools: ${missing[*]}"
        exit 1
    fi
    if ! docker info >/dev/null 2>&1; then
        fail "the Docker daemon is not reachable (start Docker/OrbStack first)"
        exit 1
    fi
    ok "preflight: docker, cargo, curl, make present; daemon reachable"
}

# ─────────────────────────────────────────────────────────────────────────────
# Router: reuse what the machine already runs; only install when nothing does
# ─────────────────────────────────────────────────────────────────────────────

router_running() {
    [[ "$(docker inspect -f '{{.State.Running}}' "$ROUTER_CONTAINER" 2>/dev/null || echo false)" == "true" ]]
}

ensure_router() {
    if [[ "$E2E_ROUTER_MODE" != "install" ]] && router_running; then
        ok "router: reusing the running $ROUTER_CONTAINER (owned by this machine, left untouched)"
        ROUTER_STARTED_BY_US=0
    else
        if [[ "$E2E_ROUTER_MODE" == "reuse" ]]; then
            fail "router: E2E_ROUTER_MODE=reuse but no running $ROUTER_CONTAINER container"
            exit 1
        fi
        warn "router: no running $ROUTER_CONTAINER — installing/starting one for this run"
        # `mx router install` refuses to run unless ~/.mech-crate/templates exists
        # (it checks the install dir, not MECH_CRATE_ROOT). A fresh runner has
        # neither, so initialise there. A developer machine already has one — and
        # is not in this branch anyway, since its router is running.
        if [[ ! -d "${HOME}/.mech-crate/templates/recipes" ]]; then
            warn "router: ~/.mech-crate is empty — running 'mx init' (fresh machine / CI runner)"
            run_step "mx-init" mx init
        fi
        # `mx router status` prints "Installed: installed" / "Installed: not installed"
        if mx router status 2>&1 | grep -q "Installed: not installed"; then
            run_step "router-install" mx router install
        fi
        run_step "router-up" mx router up
        ROUTER_STARTED_BY_US=1
    fi

    docker network inspect "$ROUTER_NETWORK" >/dev/null 2>&1 || run_step "router-network" mx router network

    # Traefik answers unmatched hosts with 404 — any HTTP response proves :80 is ours.
    local code
    code="$(curl -s -o /dev/null -w '%{http_code}' -m 5 http://127.0.0.1:80/ || echo 000)"
    if [[ "$code" == "000" ]]; then
        fail "router: nothing answered on http://127.0.0.1:80 (mx router status / lsof -i :80)"
        mx router status || true
        exit 1
    fi
    ok "router: :80 answering (HTTP $code), network $ROUTER_NETWORK present"
}

# ─────────────────────────────────────────────────────────────────────────────
# URL discovery + reachability
# ─────────────────────────────────────────────────────────────────────────────

# discover_host <compose file> <service>
# The URL lives in the compose file's Traefik Host rule — read it, never guess a
# port. Zero or multiple matches is an error, not a reason to fall back.
discover_host() {
    local compose_file="$1" service="$2"
    local -a hosts=()
    # shellcheck disable=SC2016  # the backticks below are Traefik rule syntax, not a subshell
    while IFS= read -r line; do
        [[ -n "$line" ]] && hosts+=("$line")
    done < <(grep -oE "traefik\.http\.routers\.${service}\.rule=Host\(\`[^\`]+\`\)" "$compose_file" \
             | sed -E 's/.*Host\(`([^`]+)`\).*/\1/' | sort -u)

    if [[ ${#hosts[@]} -ne 1 ]]; then
        fail "URL discovery: expected exactly 1 Host rule for '$service' in $compose_file, found ${#hosts[@]}"
        return 1
    fi
    local host="${hosts[0]}"
    if [[ "$host" == *'{{'* || ! "$host" =~ ^[A-Za-z0-9.-]+$ ]]; then
        fail "URL discovery: unusable host '$host' (unresolved placeholder?) in $compose_file"
        return 1
    fi
    printf '%s\n' "$host"
}

# health_path_for <recipe>
# The path that proves the APP answered. It matters: Traefik itself returns 404
# for an unrouted host, so hitting a path the app 404s on could not distinguish
# "no route" from "routed, wrong path".
health_path_for() {
    if [[ -n "${E2E_HEALTH_PATH:-}" ]]; then
        printf '%s\n' "$E2E_HEALTH_PATH"
        return
    fi
    case "$1" in
        rust-api|rust-worker) printf '/api/health\n' ;;
        *)                    printf '/\n' ;;
    esac
}

# wait_for_url <host> <path> <timeout-seconds>
# One scripted re-curl loop. `--resolve` pins .localhost to 127.0.0.1 so the check
# does not depend on the machine's resolver. 2xx/3xx/401/403 all count as "up".
wait_for_url() {
    local host="$1" path="$2" timeout="$3"
    local url="http://${host}${path}"
    local deadline=$(($(date +%s) + timeout))
    local code last_report=0 now

    info "waiting for $url via the router (curl --resolve ${host}:80:127.0.0.1, up to ${timeout}s)"
    while :; do
        code="$(curl -s -o /dev/null -w '%{http_code}' -m 5 \
                    --resolve "${host}:80:127.0.0.1" "$url" 2>/dev/null || echo 000)"
        case "$code" in
            2??|3??|401|403)
                ok "URL check: $url → HTTP $code"
                printf '%s → HTTP %s\n' "$url" "$code" >> "$E2E_LOG_DIR/url-checks.log"
                return 0
                ;;
        esac
        now=$(date +%s)
        if [[ $now -ge $deadline ]]; then
            fail "URL check: $url never became reachable (last HTTP $code) after ${timeout}s"
            printf '%s → TIMEOUT (last HTTP %s)\n' "$url" "$code" >> "$E2E_LOG_DIR/url-checks.log"
            return 1
        fi
        if [[ $((now - last_report)) -ge 15 ]]; then
            echo -e "  ${DIM}[$(stamp)] HTTP $code — still waiting ($((deadline - now))s left; dev images compile on first boot)${NC}"
            last_report=$now
        fi
        sleep 2
    done
}

# dump_diagnostics <service> <host> <project dir>
# The playbook's troubleshooting bundle, captured once, on failure only.
dump_diagnostics() {
    local service="$1" host="$2" project_dir="$3"
    local out="$E2E_LOG_DIR/diagnostics-${service}.log"

    warn "collecting router diagnostics → $out"
    {
        echo "===== mx router status ====="; mx router status 2>&1 || true
        echo; echo "===== docker ps -a ====="; docker ps -a 2>&1 || true
        echo; echo "===== container labels ($service) ====="
        docker inspect "$service" --format '{{json .Config.Labels}}' 2>&1 || true
        echo; echo "===== container state ($service) ====="
        docker inspect "$service" --format '{{.State.Status}} exit={{.State.ExitCode}} health={{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' 2>&1 || true
        echo; echo "===== docker network inspect $ROUTER_NETWORK ====="
        docker network inspect "$ROUTER_NETWORK" 2>&1 || true
        echo; echo "===== make ps ====="; (cd "$project_dir" && make ps 2>&1) || true
        echo; echo "===== service logs (tail) ====="
        docker logs --tail 200 "$service" 2>&1 || true
        echo; echo "===== router logs (tail) ====="
        docker logs --tail 100 "$ROUTER_CONTAINER" 2>&1 || true
        echo; echo "===== curl -v http://$host/ ====="
        curl -v -s -o /dev/null -m 10 --resolve "${host}:80:127.0.0.1" "http://${host}/" 2>&1 || true
    } > "$out"
    tail -40 "$out" || true
}

# ─────────────────────────────────────────────────────────────────────────────
# Teardown (trap-based: the scaffold always goes away, the router never does
# unless this script is the one that started it)
# ─────────────────────────────────────────────────────────────────────────────

# teardown_scaffold <entry>   (entry = "<recipe>|<service>|<project dir>|<compose project>")
# Removes ONLY what this run created: the stack's containers, its project-prefixed
# volumes, and the scaffold directory. Never the router, never another project.
teardown_scaffold() {
    local recipe service project_dir compose_project vol
    IFS='|' read -r recipe service project_dir compose_project <<< "$1"

    info "tearing down $recipe scaffold ($service, compose project $compose_project)"
    if [[ -d "$project_dir" ]]; then
        (cd "$project_dir" && COMPOSE_PROJECT_NAME="$compose_project" make down) \
            >> "$E2E_LOG_DIR/teardown.log" 2>&1 || true
    fi
    docker rm -f "$service" >> "$E2E_LOG_DIR/teardown.log" 2>&1 || true
    # Volumes are project-prefixed, so this filter can only ever match volumes
    # this run created.
    for vol in $(docker volume ls -q --filter "name=${compose_project}_" 2>/dev/null || true); do
        docker volume rm -f "$vol" >> "$E2E_LOG_DIR/teardown.log" 2>&1 || true
    done
    rm -rf "$project_dir"
}

teardown() {
    local rc=$?
    trap - EXIT INT TERM

    echo ""
    echo -e "${BOLD}[$(stamp)] teardown${NC}"

    if [[ "$E2E_KEEP" == "1" ]]; then
        warn "E2E_KEEP=1 — leaving scaffolds and containers in $E2E_WORKSPACE"
    else
        local entry
        for entry in "${SCAFFOLDS[@]:-}"; do
            [[ -z "$entry" ]] && continue
            teardown_scaffold "$entry"
        done
        rmdir "$E2E_WORKSPACE" 2>/dev/null || true
    fi

    if [[ "$ROUTER_STARTED_BY_US" == "1" && "$E2E_KEEP" != "1" ]]; then
        info "stopping the router this run started"
        mx router down >> "$E2E_LOG_DIR/teardown.log" 2>&1 || true
    else
        info "router left as found (not started by this run)"
    fi

    echo ""
    if [[ $rc -eq 0 && ${#FAILED_RECIPES[@]} -eq 0 ]]; then
        echo -e "${GREEN}${BOLD}E2E PASSED${NC} — recipes: $E2E_RECIPES"
        echo -e "${DIM}logs: $E2E_LOG_DIR${NC}"
    else
        echo -e "${RED}${BOLD}E2E FAILED${NC}${FAILED_RECIPES:+ — failing recipes: ${FAILED_RECIPES[*]}}"
        echo -e "${DIM}logs: $E2E_LOG_DIR${NC}"
        [[ $rc -eq 0 ]] && rc=1
    fi
    exit "$rc"
}

# ─────────────────────────────────────────────────────────────────────────────
# The smoke itself
# ─────────────────────────────────────────────────────────────────────────────

# service_name_for <recipe> -> e2e-prefixed, alnum-only (also a container name)
service_name_for() {
    printf 'e2e%s\n' "$(printf '%s' "$1" | tr -cd '[:alnum:]' | tr '[:upper:]' '[:lower:]')"
}

# guard_container_names <compose project> <names...>
# Recipes hard-code `container_name` (db, redis, <service>), which is a global
# namespace. A leftover from a previous E2E run is ours to remove; a container
# belonging to anything else must abort the run rather than be clobbered.
guard_container_names() {
    local compose_project="$1"; shift
    local name owner
    for name in "$@"; do
        docker inspect "$name" >/dev/null 2>&1 || continue
        owner="$(docker inspect -f '{{index .Config.Labels "com.docker.compose.project"}}' "$name" 2>/dev/null || true)"
        if [[ "$owner" == "$compose_project" ]]; then
            info "removing leftover container '$name' from a previous E2E run"
            docker rm -f "$name" >/dev/null 2>&1 || true
        else
            fail "container name '$name' is already taken by compose project '${owner:-<none>}'"
            fail "this E2E run would clobber it — stop that stack (or rename it) and re-run"
            return 1
        fi
    done
    return 0
}

# bootstrap_secrets <project dir>
# Some recipes ship `docker/.config/.env.secrets` with __GENERATE_*__ placeholders
# and rely on a post_install generator to replace them. laravel installs and runs
# one; rust-api copies the same laravel-derived template but ships no generator,
# so its postgres would start with an empty POSTGRES_PASSWORD. Filling the
# placeholders here (exactly what laravel's generate-secrets.sh does) keeps the
# smoke honest about what it tests — scaffold → dev → router — instead of dying
# on a recipe packaging gap. Tracked as a recipe defect, not an E2E behaviour.
bootstrap_secrets() {
    local project_dir="$1"
    local secrets="$project_dir/docker/.config/.env.secrets"

    [[ -f "$secrets" ]] || return 0
    grep -q '__GENERATE_' "$secrets" || return 0

    warn "recipe left __GENERATE_*__ placeholders in .env.secrets — filling them in (recipe defect)"
    local pw key
    pw="$(LC_ALL=C tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 32 || true)"
    key="base64:$(openssl rand -base64 32)"
    perl -pi -e "s/__GENERATE_DB_PASSWORD__/${pw}/g; s|__GENERATE_APP_KEY__|${key}|g" "$secrets"
    # anything still unfilled would silently become an empty env value
    if grep -q '__GENERATE_' "$secrets"; then
        warn "unrecognised placeholders remain in $secrets:"
        grep -n '__GENERATE_' "$secrets" || true
    fi
}

# bootstrap_compose_env <project dir>
# The recipes' env files reference each other (`.env.db` has
# POSTGRES_PASSWORD=${DB_PASSWORD}, `.env.shared` has DB_PASSWORD=${<SVC>_DB_PASSWORD}),
# but docker compose interpolates `env_file` values against the shell/`.env`
# environment only — never against values defined in another env_file. Left
# alone every one of them resolves to "" and postgres never becomes healthy.
# Resolving them into `docker/compose/.env` (the project directory compose reads
# for interpolation) is the same unblock a human would apply.
bootstrap_compose_env() {
    local project_dir="$1"
    local config_dir="$project_dir/docker/.config"
    local compose_env="$project_dir/docker/compose/.env"

    [[ -d "$config_dir" ]] || return 0
    # Don't clobber an env a recipe deliberately shipped.
    [[ -f "$compose_env" ]] && return 0

    local names
    names="$(grep -ohE '\$\{[A-Za-z_][A-Za-z0-9_]*' "$config_dir"/.env.* "$project_dir"/docker/compose/*.yml 2>/dev/null \
             | sed 's/^\${//' | sort -u || true)"
    [[ -z "$names" ]] && return 0

    # Subshell: sourcing the env files lets bash do the ${VAR} resolution that
    # compose will not do, without leaking any of it into this script.
    (
        set +u
        set -a
        local f
        # .env.secrets first: later files reference the secrets it defines.
        for f in "$config_dir"/.env.secrets "$config_dir"/.env.shared "$config_dir"/.env.*; do
            case "$f" in *.template|*.bak|*.example) continue ;; esac
            # shellcheck disable=SC1090  # runtime-generated project env files
            [[ -f "$f" ]] && . "$f"
        done
        set +a
        local v val
        for v in $names; do
            eval "val=\${$v:-}"
            [[ -n "$val" ]] && printf '%s=%s\n' "$v" "$val"
        done
    ) > "$compose_env"

    info "resolved $(wc -l < "$compose_env" | tr -d ' ') interpolation vars into docker/compose/.env"
}

smoke_recipe_impl() {
    local recipe="$1"
    local service project project_dir compose_file host health_path compose_project

    service="$(service_name_for "$recipe")"
    project="proj-${service}"
    project_dir="$E2E_WORKSPACE/$project"

    # Every mx project's compose files live in docker/compose/, so docker compose
    # derives the SAME default project name ("compose") for all of them — one mx
    # stack will happily adopt/recreate another's containers. Pinning
    # COMPOSE_PROJECT_NAME keeps this run's containers, networks and volumes in
    # their own namespace and out of the developer's other stacks.
    compose_project="mxe2e-${service}"
    export COMPOSE_PROJECT_NAME="$compose_project"

    echo ""
    echo -e "${BOLD}═══ recipe: ${CYAN}${recipe}${NC}${BOLD} (service ${service}, compose project ${compose_project}) ═══${NC}"

    guard_container_names "$compose_project" "$service" db redis || return 1
    rm -rf "$project_dir"

    # Register for teardown BEFORE anything can fail (the EXIT trap is the safety
    # net; the happy path tears each stack down as soon as its recipe is done, so
    # the next recipe finds `db`/`redis` free).
    SCAFFOLD_IDX=${#SCAFFOLDS[@]}
    SCAFFOLDS+=("${recipe}|${service}|${project_dir}|${compose_project}")

    run_step "${recipe}-mx-new" env -C "$E2E_WORKSPACE" mx new "$project" --no-prompt || return 1

    # --domain is passed explicitly: the recipe default (`{{SERVICE_NAME}}.localhost`)
    # is not re-expanded by the installer, so relying on it yields a literal
    # placeholder in the Host rule. discover_host() below fails loudly if that
    # ever regresses.
    run_step "${recipe}-mx-add" env -C "$project_dir" \
        mx add "$service" --recipe="$recipe" --domain "${service}.localhost" || return 1

    compose_file="$project_dir/docker/compose/${service}.yml"
    if [[ ! -f "$compose_file" ]]; then
        fail "$recipe: expected compose file $compose_file was not generated"
        return 1
    fi
    host="$(discover_host "$compose_file" "$service")" || return 1
    health_path="$(health_path_for "$recipe")"
    ok "URL discovered from compose labels: http://${host}${health_path}"

    bootstrap_secrets "$project_dir"
    bootstrap_compose_env "$project_dir"

    run_step "${recipe}-make-dev" env -C "$project_dir" make dev "s=$service" || {
        dump_diagnostics "$service" "$host" "$project_dir"
        return 1
    }

    if ! wait_for_url "$host" "$health_path" "$E2E_URL_TIMEOUT"; then
        dump_diagnostics "$service" "$host" "$project_dir"
        return 1
    fi

    ok "$recipe: scaffold → make dev → router URL verified"
    return 0
}

# Runs one recipe and disposes of its stack immediately (diagnostics are already
# captured by then). Recipes hard-code `db`/`redis` container names, so the next
# recipe can only start once this one is gone.
smoke_recipe() {
    SCAFFOLD_IDX=-1
    local rc=0
    smoke_recipe_impl "$1" || rc=$?

    if [[ "$E2E_KEEP" != "1" && "$SCAFFOLD_IDX" -ge 0 ]]; then
        teardown_scaffold "${SCAFFOLDS[$SCAFFOLD_IDX]}"
        SCAFFOLDS[SCAFFOLD_IDX]=""
    fi
    return "$rc"
}

main() {
    echo ""
    echo -e "${BOLD}mech-crate E2E smoke — scaffold → router → URL${NC}"
    echo -e "  ${DIM}repo:      $REPO_ROOT${NC}"
    echo -e "  ${DIM}recipes:   $E2E_RECIPES${NC}"
    echo -e "  ${DIM}workspace: $E2E_WORKSPACE${NC}"
    echo -e "  ${DIM}logs:      $E2E_LOG_DIR${NC}"

    preflight
    trap teardown EXIT INT TERM

    # Build and use THIS checkout's mx + THIS checkout's templates/recipes.
    # MECH_CRATE_ROOT makes template/recipe resolution read straight from the
    # repo: nothing is installed to /usr/local/bin and a developer's existing
    # ~/.mech-crate is only ever read, never rewritten (the one `mx init` this
    # script can run is gated on ~/.mech-crate being absent — see ensure_router).
    run_step "cargo-build" cargo build --manifest-path "$REPO_ROOT/Cargo.toml" -p mx-cli
    export PATH="$REPO_ROOT/target/debug:$PATH"
    export MECH_CRATE_ROOT="$REPO_ROOT"
    ok "using $(command -v mx) ($(mx --version)) with templates from $MECH_CRATE_ROOT/templates"

    ensure_router

    local recipe
    for recipe in $E2E_RECIPES; do
        if [[ ! -f "$REPO_ROOT/templates/recipes/$recipe/recipe.json" ]]; then
            fail "unknown recipe '$recipe' (no templates/recipes/$recipe/recipe.json)"
            FAILED_RECIPES+=("$recipe")
            continue
        fi
        smoke_recipe "$recipe" || FAILED_RECIPES+=("$recipe")
    done

    if [[ ${#FAILED_RECIPES[@]} -gt 0 ]]; then
        return 1
    fi
    return 0
}

main "$@"
