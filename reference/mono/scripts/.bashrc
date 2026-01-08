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

compose_context_files() {
    local dir=tmp/up
    local files=""
    local service=$1
    local add_dev=$2
    local base_file=docker/compose/${service}.yml

    if [ -f "$base_file" ]; then
        files+=" -f $base_file "
    else
        echo ""
        exit 0
    fi

    # Enable nullglob to handle no .txt files scenario
    shopt -s nullglob

    # Check if there are any .txt files in the specified directory
    local txt_files=("$dir"/*.txt)

    if [ ${#txt_files[@]} -gt 0 ]; then
        # Concatenate all .txt files' contents
        files+=$(cat "$dir"/*.txt)
    fi

    # Disable nullglob after use
    shopt -u nullglob

    if [ "$add_dev" = "true" ]; then
        if [ -f "docker/compose/${service}-dev.yml" ]; then
            files+=" -f docker/compose/${service}-dev.yml"
        fi
    fi

    # Check if a compose file exists for the current processor architecture
    if [ -f "docker/compose/$(uname -m).yml" ]; then
        # If a compose file exists, append it to the list of files
        files+=" -f docker/compose/$(uname -m).yml"
    fi

    # Return the concatenated contents or empty string
    echo "$files"
}

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

    #echo $files

    # Create a temporary directory to store modified compose files
    tmp_dir="docker/.compose"
    mkdir -p $tmp_dir

    rm -rf $tmp_dir/*
    cp -rf docker/compose/* $tmp_dir/ >/dev/null 2>&1
    cp -f docker/compose/.env $tmp_dir/

    # Replace new line characters with space
    files=$(echo "$files" | awk '{printf "%s ", $0}')

    file_array=()
    while IFS= read -r -d ' ' entry; do
        file_array+=("$entry")
    done <<<"$files "

    # Get the IP address of the Traefik container
    TRAEFIK_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' traefik)

    # Check if the IP address was found
    if [ -z "$TRAEFIK_IP" ]; then
        echo "docker compose $files up -d $service with no traefik"
        dt=$(date '+%d%m%Y%H%M%S')
        echo ${file_array[@]} >tmp/up/up-$dt.txt
        docker compose $files up -d $service
        exit 0
    fi

    extra_hosts_entry="- \"connect.localhost:$TRAEFIK_IP\""

    # Process each compose file
    for i in "${!file_array[@]}"; do
        if [[ ${file_array[$i]} == "-f" ]]; then
            original_file=${file_array[$((i + 1))]}
            modified_file="$tmp_dir/$(basename ${original_file})"

            # Copy the original file to the temporary directory
            cp -f $original_file $modified_file 2>/dev/null

            # Replace the placeholder with the actual extra_hosts entry if it exists
            if grep -q "\- \"placeholder:127.0.0.1\"" "$modified_file"; then
                if [[ "$(uname)" == "Darwin" ]]; then
                    # macOS/BSD sed
                    sed -i '' "s|\- \"placeholder:127.0.0.1\"|$extra_hosts_entry|" "$modified_file"
                else
                    # GNU sed
                    sed -i "s|\- \"placeholder:127.0.0.1\"|$extra_hosts_entry|" "$modified_file"
                fi
            fi

            # Update the file_array to use the modified file
            file_array[$((i + 1))]=$modified_file
        fi
    done

    ##echo ">>>> ${file_array[@]}"

    deduplicated_files=$(deduplicate_services "${file_array[@]}")

    if [ -z "$deduplicated_files" ]; then
        echo "dedupe failed for $service"
        echo ">>>> ${file_array[@]}"
        echo ">>>> $deduplicated_files"
        exit 1
    fi

    dt=$(date '+%d%m%Y%H%M%S')
    rm -f tmp/up/up-*.txt
    echo $deduplicated_files >tmp/up/up-$dt.txt

    # Start the specified service using the modified compose files
    echo "docker compose $deduplicated_files up -d $service"

    docker compose $deduplicated_files up -d $service
}

is_mac() {
  if [[ "$OSTYPE" == "darwin"* ]]; then
    return 0  # True (macOS)
  else
    return 1  # False (Not macOS)
  fi
}

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

    if grep -q "^${env_var_name}_IMAGE_TAG=" "$env_file"; then
       # OS-specific sed behavior: Use `OSTYPE` to detect macOS or Linux
       if is_mac; then
         # macOS (BSD sed)
         sed -i '' "s/^${env_var_name}_IMAGE_TAG=.*/${full_env_var}/" "$env_file"
       else
         # Linux (GNU sed)
         sed -i "s/^${env_var_name}_IMAGE_TAG=.*/${full_env_var}/" "$env_file"
       fi
       echo "Updated ${env_var_name}_IMAGE_TAG in $env_file"
     else
       # If it doesn't exist, append the key=value pair to the end of the file
       echo "${full_env_var}" >> "$env_file"
       echo "Added ${env_var_name}_IMAGE_TAG to $env_file"
     fi
}

convert_service_to_env_var() {
  local service_name="$1"

  # Convert service name to uppercase and replace dashes with underscores
  env_var_name=$(echo "$service_name" | tr '[:lower:]' '[:upper:]' | sed 's/[-.]/_/g')

  # Return the formatted environment variable name (without _IMAGE_TAG)
  echo "${env_var_name}"
}
