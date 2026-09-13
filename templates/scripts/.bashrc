# MechCrate Helper Functions
# Source this file in scripts: source ./scripts/.bashrc

# ─────────────────────────────────────────────────────────────────────────────
# Compose project isolation
#
# THIS FILE IS THE AUTHORITATIVE SOURCE of the docker compose project name for
# this project. Every script that shells out to `docker compose` sources this
# file and passes `-p "$COMPOSE_PROJECT_NAME"`.
#
# Why it has to be pinned: every mx project keeps its compose files in
# docker/compose/, and `docker compose` derives its default project name from
# the compose file's parent directory — i.e. "compose" for EVERY mx project on
# the machine. Two projects then share one namespace and adopt/recreate each
# other's containers, networks and volumes (observed live: an e2e scaffold
# recreated an unrelated project's `db`).
#
# Alternatives considered and rejected:
#   * a top-level `name:` key in the compose files — compose-native, and it
#     would also cover hand-rolled `docker compose` calls, but the shared
#     docker/compose/*.yml are copied verbatim (no placeholder expansion) and
#     every recipe fragment would need its own copy, so the name would live in
#     N places and drift.
#   * COMPOSE_PROJECT_NAME in docker/.config/.env.shared — that file is an
#     `env_file:` for the *containers*; the compose CLI never reads it.
#
# Known limitation: a hand-rolled `docker compose -f docker/compose/x.yml …`
# that does not source this file still gets the default name. `make doctor`
# warns when it finds containers for this project's services under a different
# compose project name.
# ─────────────────────────────────────────────────────────────────────────────

# Absolute path of the project root (the directory holding scripts/).
# Derived from this file's location, so it is correct regardless of the caller's
# working directory; falls back to $PWD in shells without BASH_SOURCE.
mech_project_root() {
    if [ -n "${BASH_SOURCE[0]:-}" ]; then
        (cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
    else
        pwd
    fi
}

# Derive this project's compose project name from its directory name.
# Usage: mech_compose_project_name [root_dir]
#
# Compose project names must match [a-z0-9][a-z0-9_-]*, so the directory name is
# lowercased, every other character becomes '-', and leading non-alphanumerics
# are stripped. Collision note: two directories with the SAME name on one
# machine still derive the same project name — accepted; name project
# directories uniquely (or export COMPOSE_PROJECT_NAME yourself).
mech_compose_project_name() {
    local root raw name
    root="${1:-$(mech_project_root)}"
    raw="$(basename "$root")"
    name="$(printf '%s' "$raw" |
        tr '[:upper:]' '[:lower:]' |
        sed -e 's/[^a-z0-9_-]/-/g' -e 's/^[^a-z0-9]*//')"
    [ -n "$name" ] || name="mx-project"
    printf '%s\n' "$name"
}

# Pin the name for every compose invocation in this project. An explicit value
# already in the environment always wins, so harnesses (e.g. an e2e runner) can
# namespace their own runs.
: "${COMPOSE_PROJECT_NAME:=$(mech_compose_project_name)}"
export COMPOSE_PROJECT_NAME

# ─────────────────────────────────────────────────────────────────────────────
# Dev credentials
#
# THIS FILE IS THE AUTHORITATIVE SOURCE of what counts as an unset credential,
# which keys are deliberately left blank, and how a generated dev value is
# shaped. `scripts/generate-secrets.sh` writes the values; `scripts/doctor.sh`
# reports what is still missing. Both read the rules from here so they can never
# disagree about whether a project is healthy.
# ─────────────────────────────────────────────────────────────────────────────

# Credentials the shipped stack deliberately leaves blank.
#
# The bundled redis runs `redis-server` with no `--requirepass`, so a generated
# REDIS_PASSWORD would break every client that builds
# `redis://:$REDIS_PASSWORD@redis:6379` against a server that wants no auth.
# Blank is the correct dev value, and doctor must not nag about it.
MECH_BLANK_BY_DESIGN_KEYS="REDIS_PASSWORD"

# True when a value is not a real credential: empty, or still one of the
# placeholder conventions the templates ship.
# Usage: mech_secret_is_unset "<value>"
mech_secret_is_unset() {
    local value="$1"
    [ -n "$value" ] || return 0
    case "$value" in
        __GENERATE_*__ | CHANGE_ME* | changeme | your-*-here) return 0 ;;
    esac
    return 1
}

# True when this key is meant to stay blank (see MECH_BLANK_BY_DESIGN_KEYS).
# Usage: mech_secret_is_blank_by_design "<key>"
mech_secret_is_blank_by_design() {
    local key="$1" candidate
    for candidate in $MECH_BLANK_BY_DESIGN_KEYS; do
        [ "$key" = "$candidate" ] && return 0
    done
    return 1
}

# Classify a key so the generator knows what shape of value it needs.
# Echoes one of: db_user | db_name | app_key | random | none.
# "none" means "not a credential" — the generator leaves it alone rather than
# filling an unrelated empty setting with 32 random characters.
mech_secret_kind() {
    case "$1" in
        APP_KEY | *_APP_KEY) echo app_key ;;
        DB_USER | *_DB_USER) echo db_user ;;
        DB_NAME | *_DB_NAME) echo db_name ;;
        *PASSWORD* | *PASSWD* | *SECRET* | *TOKEN* | *SALT* | *KEY) echo random ;;
        *) echo none ;;
    esac
}

# A random alphanumeric dev secret. Never a fixed default: a password shipped in
# the templates is the same password on every machine that ever ran `mx new`.
# Usage: mech_random_secret [length]
mech_random_secret() {
    local len="${1:-32}"
    if command -v openssl >/dev/null 2>&1; then
        openssl rand -hex "$(((len + 1) / 2))" | cut -c "1-${len}"
    else
        # dd bounds the read so `tr` never takes SIGPIPE from a downstream
        # `head` — which, under `set -o pipefail`, would fail the whole script.
        # 256 bytes yields ~150 alphanumerics, comfortably more than needed.
        dd if=/dev/urandom bs=1 count=256 2>/dev/null |
            LC_ALL=C tr -dc 'A-Za-z0-9' | cut -c "1-${len}"
    fi
}

# A Laravel-style application key. Harmless for recipes that ignore APP_KEY.
mech_random_app_key() {
    if command -v openssl >/dev/null 2>&1; then
        printf 'base64:%s\n' "$(openssl rand -base64 32)"
    else
        printf 'base64:%s\n' "$(mech_random_secret 44)"
    fi
}

# A postgres-safe identifier derived from the project directory, for the db role
# and database name. Postgres folds unquoted identifiers to lower case and
# rejects a leading digit, so: lowercase, non-alphanumerics to '_', digit-leading
# names prefixed.
mech_db_identifier() {
    local name
    name="$(mech_compose_project_name "${1:-}" | tr '-' '_')"
    case "$name" in
        [0-9]*) name="db_$name" ;;
    esac
    printf '%s\n' "$name"
}

# Keys in an env file whose value is still unset (empty or a placeholder),
# excluding the ones that are blank by design. One key per line.
# Usage: mech_unset_secret_keys <env-file>
mech_unset_secret_keys() {
    local file="$1" line key value
    [ -f "$file" ] || return 0
    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            '' | '#'*) continue ;;
            *=*) ;;
            *) continue ;;
        esac
        key="${line%%=*}"
        value="${line#*=}"
        mech_secret_is_blank_by_design "$key" && continue
        if mech_secret_is_unset "$value"; then
            printf '%s\n' "$key"
        fi
    done < "$file"
}

# Deduplicate compose file arguments
# Prevents duplicate -f flags when composing multiple services
deduplicate_services() {
    local input="$@"
    local -a seen
    local -a file_array
    local result=""

    # Split the input string into an array
    while IFS= read -r -d ' ' entry; do
        file_array+=("$entry")
    done <<<"$input "

    # Iterate over the split arguments
    for ((i = 0; i < ${#file_array[@]}; i++)); do
        if [[ "${file_array[i]}" == "-f" ]]; then
            full_arg="${file_array[i]} ${file_array[i + 1]}"
            if [[ ! " ${seen[*]} " =~ " ${full_arg} " ]]; then
                seen+=("$full_arg")
                result="$result $full_arg"
            fi
            # Skip the next argument which is part of the "-f"
            ((i++))
        fi
    done

    echo "$result"
}

# ─────────────────────────────────────────────────────────────────────────────
# Service selection
#
# `s=` takes ONE service or a whitespace-separated list (`make dev s="api site"`)
# and arrives here as a single argument — the make layer quotes it, or the second
# name would become a make goal (bd:mech-crate-3kq).
#
# Which targets take a list is decided by the verb underneath, not by taste:
#
#   dev, up, down, stop, restart, logs   list  — compose takes N service operands
#   build, run, exec, sh/bash            one   — one image / one container
#
# The single-service targets refuse a list out loud (mech_require_single_service)
# rather than quietly acting on the first name.
# ─────────────────────────────────────────────────────────────────────────────

# Append " -f <file> " to a compose-file argument string, at most once.
# Two selected services that both pull in db.yml (or a re-selected service
# already in the saved context) would otherwise repeat it: compose tolerates the
# repetition, but every command line mx echoes would carry the noise.
# Usage: files=$(mech_append_compose_file "$files" docker/compose/db.yml)
mech_append_compose_file() {
    local files="$1" file="$2"
    case " $files " in
        *" -f $file "*) printf '%s' "$files" ;;
        *) printf '%s -f %s ' "$files" "$file" ;;
    esac
}

# Refuse a service list for a target that acts on exactly one image/container.
# Silently taking the first name is the failure this exists to prevent.
# Usage: mech_require_single_service "$1" "make build" || exit 1
mech_require_single_service() {
    local value="$1" label="${2:-this command}"
    # Intentionally unquoted: this is the split that counts the names.
    set -- $value
    [ "$#" -le 1 ] && return 0

    print_error "$label takes a single service only - got $# ('$value')"
    echo "  Run it once per service, e.g.: $label s=$1"
    return 1
}

# Get compose files for one service, a service list, or the whole project
# Usage: compose_context_files "service [service ...]" "add_dev"
# Returns: -f file1.yml -f file2.yml ...
#
# Every requested name contributes its own docker/compose/<name>.yml (plus
# <name>.dev.yml when add_dev is true). A name with no compose file is reported
# by name and the whole context comes back empty — starting the subset that
# happens to resolve would be worse than refusing.
compose_context_files() {
    local dir=tmp/up
    local files=""
    local add_dev=$2
    local service=""
    local missing=""

    # Intentionally unquoted: one or more service names arrive as a single
    # whitespace-separated argument, and this is the split. No names at all (an
    # empty or whitespace-only s=) means "the whole project", which is how make
    # treats it too.
    set -- ${1:-}

    # If no service provided, build a context across all base compose files.
    if [ "$#" -eq 0 ]; then
        shopt -s nullglob
        local base_files=(docker/compose/*.yml)
        shopt -u nullglob

        # Filter out dev overrides + arch-specific file (added later)
        local arch_file="docker/compose/$(uname -m).yml"
        local found_any="false"
        for f in "${base_files[@]}"; do
            [[ "$f" == *".dev.yml" ]] && continue
            [[ "$f" == "$arch_file" ]] && continue
            [[ "$f" == "docker/compose/.env" ]] && continue
            if [[ -f "$f" ]]; then
                files+=" -f $f "
                found_any="true"
            fi
        done

        if [[ "$found_any" != "true" ]]; then
            echo ""
            return 0
        fi

        if [[ "$add_dev" == "true" ]]; then
            shopt -s nullglob
            local dev_files=(docker/compose/*.dev.yml)
            shopt -u nullglob
            for f in "${dev_files[@]}"; do
                [[ -f "$f" ]] && files+=" -f $f "
            done
        fi

        # Add arch-specific override if present
        if [ -f "$arch_file" ]; then
            files+=" -f $arch_file "
        fi

        echo "$files"
        return 0
    fi

    # Every requested name must resolve, or nothing does. Report each miss by
    # name: "no service configuration found" for the whole list leaves the
    # caller guessing which of them was the typo.
    for service in "$@"; do
        if [ ! -f "docker/compose/${service}.yml" ]; then
            missing="$missing $service"
        fi
    done
    if [ -n "$missing" ]; then
        for service in $missing; do
            echo "No compose file for service '$service' (docker/compose/${service}.yml)" >&2
        done
        echo ""
        return 0
    fi

    for service in "$@"; do
        files=$(mech_append_compose_file "$files" "docker/compose/${service}.yml")
    done

    # Fold in the context saved by previous runs (tmp/up/*.txt), one -f pair at a
    # time so a file already selected above is not repeated.
    shopt -s nullglob
    local ctx_file token prev=""
    for ctx_file in "$dir"/*.txt; do
        prev=""
        # Intentionally unquoted: the saved context is a flat "-f a.yml -f b.yml".
        for token in $(cat "$ctx_file"); do
            if [ "$prev" = "-f" ]; then
                files=$(mech_append_compose_file "$files" "$token")
            fi
            prev="$token"
        done
    done
    shopt -u nullglob

    # Add dev override files if requested
    if [ "$add_dev" = "true" ]; then
        for service in "$@"; do
            if [ -f "docker/compose/${service}.dev.yml" ]; then
                files=$(mech_append_compose_file "$files" "docker/compose/${service}.dev.yml")
            fi
        done
    fi

    # Check if a compose file exists for the current processor architecture
    if [ -f "docker/compose/$(uname -m).yml" ]; then
        files=$(mech_append_compose_file "$files" "docker/compose/$(uname -m).yml")
    fi

    # Return the concatenated contents
    echo "$files"
}

# Run a service with its compose context
# Usage: run_service_in_context "$files" "$service"
run_service_in_context() {
    local files=$1
    local service=$2

    # Check if services are found
    if [ -n "$files" ]; then
        echo "Services found..."
    else
        echo "No services found"
        exit 1
    fi

    # Create temporary directory
    tmp_dir="docker/.compose"
    mkdir -p $tmp_dir

    rm -rf $tmp_dir/*
    cp -rf docker/compose/* $tmp_dir/ >/dev/null 2>&1

    # Copy .env if it exists
    if [ -f "docker/compose/.env" ]; then
        cp -f docker/compose/.env $tmp_dir/
    fi

    # Replace new line characters with space
    files=$(echo "$files" | awk '{printf "%s ", $0}')

    file_array=()
    while IFS= read -r -d ' ' entry; do
        file_array+=("$entry")
    done <<<"$files "

    deduplicated_files=$(deduplicate_services "${file_array[@]}")

    if [ -z "$deduplicated_files" ]; then
        echo "Dedupe failed for $service"
        echo ">>>> ${file_array[@]}"
        exit 1
    fi

    # Save context for later use (logs, down, etc.)
    dt=$(date '+%d%m%Y%H%M%S')
    rm -f tmp/up/up-*.txt
    echo $deduplicated_files > tmp/up/up-$dt.txt

    # Start the service(s) — always under this project's own compose project
    # name, never the directory-derived default ("compose") every mx project
    # would otherwise share.
    if [[ -n "${service:-}" ]]; then
        echo "docker compose -p $COMPOSE_PROJECT_NAME $deduplicated_files up -d $service"
        docker compose -p "$COMPOSE_PROJECT_NAME" $deduplicated_files up -d $service
    else
        echo "docker compose -p $COMPOSE_PROJECT_NAME $deduplicated_files up -d"
        docker compose -p "$COMPOSE_PROJECT_NAME" $deduplicated_files up -d
    fi
}

# Check if running on macOS
is_mac() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        return 0
    else
        return 1
    fi
}

# Set environment variable in compose .env file
# Usage: setenv "VAR_NAME" "value"
setenv() {
    local env_var_name="$1"
    local tag_value="$2"
    local env_file="docker/compose/.env"

    if [ -z "$env_var_name" ]; then
        echo "No environment variable name provided"
        return 1
    fi

    if [ -z "$tag_value" ]; then
        echo "No tag value provided"
        return 1
    fi

    full_env_var="${env_var_name}_IMAGE_TAG=${tag_value}"

    if [ ! -f "$env_file" ]; then
        echo "${full_env_var}" > "$env_file"
        echo "Created $env_file with ${env_var_name}_IMAGE_TAG"
        return 0
    fi

    if grep -q "^${env_var_name}_IMAGE_TAG=" "$env_file"; then
        if is_mac; then
            sed -i '' "s/^${env_var_name}_IMAGE_TAG=.*/${full_env_var}/" "$env_file"
        else
            sed -i "s/^${env_var_name}_IMAGE_TAG=.*/${full_env_var}/" "$env_file"
        fi
        echo "Updated ${env_var_name}_IMAGE_TAG in $env_file"
    else
        echo "${full_env_var}" >> "$env_file"
        echo "Added ${env_var_name}_IMAGE_TAG to $env_file"
    fi
}

# Convert service name to environment variable format
# Usage: convert_service_to_env_var "my-service" -> MY_SERVICE
convert_service_to_env_var() {
    local service_name="$1"
    env_var_name=$(echo "$service_name" | tr '[:lower:]' '[:upper:]' | sed 's/[-.]/_/g')
    echo "${env_var_name}"
}

# Print colored output
print_info() {
    echo -e "\033[0;34mℹ\033[0m $1"
}

print_success() {
    echo -e "\033[0;32m✓\033[0m $1"
}

print_warn() {
    echo -e "\033[1;33m⚠\033[0m $1"
}

print_error() {
    echo -e "\033[0;31m✗\033[0m $1"
}
