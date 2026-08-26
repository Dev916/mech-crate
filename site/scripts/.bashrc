# MechCrate Helper Functions
# Source this file in scripts: source ./scripts/.bashrc

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

# Get compose files for a service
# Usage: compose_context_files "service" "add_dev"
# Returns: -f file1.yml -f file2.yml ...
compose_context_files() {
    local dir=tmp/up
    local files=""
    local service=$1
    local add_dev=$2
    local base_file=""

    # If no service provided, build a context across all base compose files.
    if [[ -z "${service:-}" ]]; then
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

    base_file="docker/compose/${service}.yml"

    # Check if base file exists for the requested service
    if [ -f "$base_file" ]; then
        files+=" -f $base_file "
    else
        echo ""
        return 0
    fi

    # Enable nullglob to handle no .txt files scenario
    shopt -s nullglob

    # Check if there are any .txt files in the specified directory
    local txt_files=("$dir"/*.txt)

    if [ ${#txt_files[@]} -gt 0 ]; then
        # Concatenate all .txt files' contents (existing context)
        files+=$(cat "$dir"/*.txt)
    fi

    # Disable nullglob after use
    shopt -u nullglob

    # Add dev override file if requested
    if [ "$add_dev" = "true" ]; then
        if [ -f "docker/compose/${service}.dev.yml" ]; then
            files+=" -f docker/compose/${service}.dev.yml"
        fi
    fi

    # Check if a compose file exists for the current processor architecture
    if [ -f "docker/compose/$(uname -m).yml" ]; then
        files+=" -f docker/compose/$(uname -m).yml"
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

    # Start the service(s)
    if [[ -n "${service:-}" ]]; then
        echo "docker compose $deduplicated_files up -d $service"
        docker compose $deduplicated_files up -d $service
    else
        echo "docker compose $deduplicated_files up -d"
        docker compose $deduplicated_files up -d
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
